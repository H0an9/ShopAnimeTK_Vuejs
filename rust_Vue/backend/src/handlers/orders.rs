use super::{ApiError, ApiResult};
use crate::{
    email::send_best_effort,
    models::{
        OrderDetail, OrderItem, OrderRow, OrderStatusHistory, PageResponse, UpdateOrderStatus,
    },
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::AssertSqlSafe;

#[derive(Deserialize)]
struct OrderListQuery {
    page: Option<i64>,
    page_size: Option<i64>,
    q: Option<String>,
    status: Option<String>,
    payment: Option<String>,
    sort: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/:id", get(get_one))
        .route("/:id/status", post(update_status))
}

fn order_select() -> &'static str {
    r#"
    SELECT TRIM(h.mahd) AS mahd, h.ngaylap, h.diachi, h.thanhtien, h.htthanhtoan,
           TRIM(h.mand) AS mand,
           nd.ten AS tenkh,
           TRIM(h.makm) AS makm,
           latest.tentrangthai AS trangthai,
           TRIM(latest.matt) AS matt,
           latest.ngaycapnhat
    FROM hoadon h
    LEFT JOIN nguoidung nd ON TRIM(nd.mand) = TRIM(h.mand)
    LEFT JOIN LATERAL (
        SELECT c.matt, t.tentrangthai, c.ngaycapnhat
        FROM cttrangthai c
        LEFT JOIN trangthai t ON TRIM(t.matt) = TRIM(c.matt)
        WHERE TRIM(c.mahd) = TRIM(h.mahd)
        ORDER BY c.ngaycapnhat DESC NULLS LAST, c.ctid DESC
        LIMIT 1
    ) latest ON true
    "#
}

fn format_vnd(value: f64) -> String {
    let mut digits = format!("{:.0}", value.max(0.0));
    let mut formatted = String::new();
    while digits.len() > 3 {
        let tail = digits.split_off(digits.len() - 3);
        formatted = if formatted.is_empty() {
            tail
        } else {
            format!("{tail}.{formatted}")
        };
    }
    if formatted.is_empty() {
        format!("{digits} VND")
    } else {
        format!("{digits}.{formatted} VND")
    }
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<OrderListQuery>,
) -> ApiResult<Json<PageResponse<OrderRow>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(5, 100);
    let q = query.q.unwrap_or_default();
    let status = query.status.unwrap_or_default();
    let payment = query.payment.unwrap_or_default();
    let order = match query.sort.as_deref() {
        Some("oldest") => "h.ngaylap ASC NULLS LAST, h.mahd ASC",
        Some("amount_desc") => "h.thanhtien DESC NULLS LAST",
        Some("amount_asc") => "h.thanhtien ASC NULLS LAST",
        _ => "h.ngaylap DESC NULLS LAST, h.mahd DESC",
    };
    let conditions = r#" WHERE ($1='' OR h.mahd ILIKE '%'||$1||'%' OR nd.ten ILIKE '%'||$1||'%' OR h.diachi ILIKE '%'||$1||'%') AND ($2='' OR TRIM(latest.matt)=$2) AND ($3='' OR h.htthanhtoan=$3) "#;
    let count_sql = format!("SELECT COUNT(*) FROM ({}) filtered WHERE ($1='' OR mahd ILIKE '%'||$1||'%' OR tenkh ILIKE '%'||$1||'%' OR diachi ILIKE '%'||$1||'%') AND ($2='' OR matt=$2) AND ($3='' OR htthanhtoan=$3)", order_select());
    let total = sqlx::query_scalar::<_, i64>(AssertSqlSafe(count_sql))
        .bind(&q)
        .bind(&status)
        .bind(&payment)
        .fetch_one(&state.pool)
        .await?;
    let sql = format!(
        "{} {} ORDER BY {} LIMIT $4 OFFSET $5",
        order_select(),
        conditions,
        order
    );
    let rows = sqlx::query_as::<_, OrderRow>(AssertSqlSafe(sql))
        .bind(&q)
        .bind(&status)
        .bind(&payment)
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(PageResponse::new(rows, total, page, page_size)))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<OrderDetail>> {
    let sql = format!("{} WHERE TRIM(h.mahd) = TRIM($1)", order_select());
    let order = sqlx::query_as::<_, OrderRow>(AssertSqlSafe(sql))
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;
    let items = sqlx::query_as::<_, OrderItem>(
        r#"
        SELECT TRIM(c.masp) AS masp, s.tensp, c.gia, c.soluong
        FROM cthoadon c
        LEFT JOIN sanpham s ON TRIM(s.masp) = TRIM(c.masp)
        WHERE TRIM(c.mahd) = TRIM($1)
        ORDER BY c.masp
        "#,
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;
    let status_history = sqlx::query_as::<_, OrderStatusHistory>(
        r#"
        SELECT TRIM(c.matt) AS matt, t.tentrangthai, c.ngaycapnhat
        FROM cttrangthai c
        LEFT JOIN trangthai t ON TRIM(t.matt) = TRIM(c.matt)
        WHERE TRIM(c.mahd) = TRIM($1)
        ORDER BY c.ngaycapnhat DESC NULLS LAST, c.ctid DESC
        "#,
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(OrderDetail {
        order,
        items,
        status_history,
    }))
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateOrderStatus>,
) -> ApiResult<Json<OrderRow>> {
    let status_id = input.matt.trim();
    if status_id.is_empty() {
        return Err(ApiError::bad_request("Vui lòng chọn trạng thái"));
    }

    let mut transaction = state.pool.begin().await?;
    let stored_order_id =
        sqlx::query_scalar::<_, String>("SELECT mahd FROM hoadon WHERE TRIM(mahd) = TRIM($1)")
            .bind(&id)
            .fetch_optional(&mut *transaction)
            .await?;
    let stored_order_id =
        stored_order_id.ok_or_else(|| ApiError::not_found("Không tìm thấy hóa đơn"))?;

    let stored_status = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT matt, tentrangthai FROM trangthai WHERE TRIM(matt) = TRIM($1)",
    )
    .bind(status_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let (stored_status_id, stored_status_name) =
        stored_status.ok_or_else(|| ApiError::bad_request("Trạng thái không hợp lệ"))?;
    let new_status_id = stored_status_id.trim().to_string();
    let new_status_name = stored_status_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(new_status_id.as_str())
        .to_string();

    let current_status = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT TRIM(matt)
        FROM cttrangthai
        WHERE TRIM(mahd) = TRIM($1)
        ORDER BY ngaycapnhat DESC NULLS LAST, ctid DESC
        LIMIT 1
        "#,
    )
    .bind(&stored_order_id)
    .fetch_optional(&mut *transaction)
    .await?
    .flatten();
    if current_status.as_deref() == Some(status_id) {
        return Err(ApiError::bad_request("Hóa đơn đang ở trạng thái này"));
    }

    if let Some(current_status_id) = current_status.as_deref() {
        let ordered_statuses =
            sqlx::query_scalar::<_, String>("SELECT TRIM(matt) FROM trangthai ORDER BY matt")
                .fetch_all(&mut *transaction)
                .await?;
        let current_position = ordered_statuses
            .iter()
            .position(|item| item == current_status_id);
        let new_position = ordered_statuses.iter().position(|item| item == status_id);
        if matches!((current_position, new_position), (Some(current), Some(new)) if new < current) {
            return Err(ApiError::bad_request(
                "Không thể chuyển hóa đơn về trạng thái trước đó",
            ));
        }
    }

    sqlx::query("INSERT INTO cttrangthai (mahd, matt, ngaycapnhat) VALUES ($1, $2, NOW() AT TIME ZONE 'Asia/Ho_Chi_Minh')")
        .bind(&stored_order_id)
        .bind(&stored_status_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;

    let sql = format!("{} WHERE TRIM(h.mahd) = TRIM($1)", order_select());
    let order = sqlx::query_as::<_, OrderRow>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    let previous_status_id = current_status
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    let new_status_code = new_status_id.trim().to_ascii_uppercase();
    let send_confirmed_notice = previous_status_id == "TT001"
        && matches!(new_status_code.as_str(), "TT002" | "TT003" | "TT004");
    let send_cancelled_notice = new_status_code == "TT005";

    if send_confirmed_notice || send_cancelled_notice {
        let customer = sqlx::query_as::<_, (Option<String>, Option<String>)>(
            r#"
            SELECT nd.email, nd.ten
            FROM hoadon h
            LEFT JOIN nguoidung nd ON TRIM(nd.mand) = TRIM(h.mand)
            WHERE TRIM(h.mahd) = TRIM($1)
            "#,
        )
        .bind(&order.mahd)
        .fetch_optional(&state.pool)
        .await?;

        if let Some((email, name)) = customer {
            if email
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
            {
                println!(
                    "Order {} has no customer email; skipped order notice",
                    order.mahd
                );
            }
            let customer_name = name.as_deref().unwrap_or("ban");
            let total = format_vnd(order.thanhtien.unwrap_or(0.0));
            let payment = order.htthanhtoan.as_deref().unwrap_or("Chưa cập nhật");
            let address = order.diachi.as_deref().unwrap_or("Chưa cập nhật");
            let subject = if send_cancelled_notice {
                format!("Đơn hàng {} đã bị hủy", order.mahd)
            } else {
                format!("Đơn hàng {} đã được xác nhận", order.mahd)
            };
            let body = if send_cancelled_notice {
                format!(
                    "Xin chào {},\n\nShop Anime TK rất tiếc phải thông báo đơn hàng {} của bạn đã bị hủy.\n\nThông tin đơn hàng:\n- Trạng thái hiện tại: {}\n- Tổng tiền: {}\n- Hình thức thanh toán: {}\n- Địa chỉ nhận hàng: {}\n\nNếu bạn cần hỗ trợ thêm, vui lòng liên hệ Shop Anime TK để được kiểm tra và xử lý nhanh nhất.\n\nTrân trọng,\nShop Anime TK",
                    customer_name,
                    order.mahd,
                    new_status_name,
                    total,
                    payment,
                    address
                )
            } else {
                format!(
                    "Xin chào {},\n\nCảm ơn bạn đã mua hàng tại Shop Anime TK. Đơn hàng {} của bạn đã được xác nhận và đang được shop xử lý.\n\nThông tin đơn hàng:\n- Trạng thái hiện tại: {}\n- Tổng tiền: {}\n- Hình thức thanh toán: {}\n- Địa chỉ nhận hàng: {}\n\nShop sẽ tiếp tục cập nhật khi đơn hàng có thay đổi. Cảm ơn bạn đã tin tưởng và ủng hộ Shop Anime TK.\n\nTrân trọng,\nShop Anime TK",
                    customer_name,
                    order.mahd,
                    new_status_name,
                    total,
                    payment,
                    address
                )
            };
            send_best_effort(email.as_deref(), &subject, &body).await;
        }
    }

    Ok(Json(order))
}
