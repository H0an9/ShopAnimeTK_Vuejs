use super::ApiResult;
use crate::{
    models::{DashboardRevenuePoint, DashboardSlice, DashboardStats, DashboardVisuals, OrderRow},
    AppState,
};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct VisualsQuery {
    start_month: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/stats", get(stats))
        .route("/visuals", get(visuals))
}

async fn stats(State(state): State<AppState>) -> ApiResult<Json<DashboardStats>> {
    let row = sqlx::query_as::<_, DashboardStats>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM sanpham) AS total_products,
            (SELECT COUNT(*) FROM nguoidung) AS total_users,
            (SELECT COUNT(*) FROM hoadon) AS total_orders,
            COALESCE((SELECT SUM(thanhtien) FROM hoadon), 0)::float8 AS total_revenue
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(row))
}

async fn visuals(
    State(state): State<AppState>,
    Query(query): Query<VisualsQuery>,
) -> ApiResult<Json<DashboardVisuals>> {
    let start_month = query.start_month.unwrap_or_default();
    let revenue = sqlx::query_as::<_, DashboardRevenuePoint>(
        r#"
        WITH bounds AS (
            SELECT
                date_trunc('month', CURRENT_DATE) - INTERVAL '11 months' AS min_month,
                date_trunc('month', CURRENT_DATE) - INTERVAL '5 months' AS default_start,
                date_trunc('month', CURRENT_DATE) - INTERVAL '5 months' AS max_start
        ),
        selected_range AS (
            SELECT LEAST(
                GREATEST(
                    CASE
                        WHEN $1 = '' THEN bounds.default_start
                        ELSE date_trunc('month', to_date($1 || '-01', 'YYYY-MM-DD'))
                    END,
                    bounds.min_month
                ),
                bounds.max_start
            ) AS start_month
            FROM bounds
        ),
        months AS (
            SELECT generate_series(
                selected_range.start_month,
                selected_range.start_month + INTERVAL '5 months',
                INTERVAL '1 month'
            ) AS month_start
            FROM selected_range
        )
        SELECT to_char(months.month_start, 'MM/YYYY') AS label,
               COALESCE(SUM(h.thanhtien), 0)::float8 AS total
        FROM months
        LEFT JOIN hoadon h
          ON date_trunc('month', h.ngaylap::timestamp) = months.month_start
        GROUP BY months.month_start
        ORDER BY months.month_start
        "#,
    )
    .bind(start_month)
    .fetch_all(&state.pool)
    .await?;

    let order_statuses = sqlx::query_as::<_, DashboardSlice>(
        r#"
        SELECT COALESCE(latest.tentrangthai, 'Chưa có trạng thái') AS label,
               COUNT(*)::bigint AS value
        FROM hoadon h
        LEFT JOIN LATERAL (
            SELECT t.tentrangthai
            FROM cttrangthai c
            LEFT JOIN trangthai t ON t.matt = c.matt
            WHERE c.mahd = h.mahd
            ORDER BY c.ngaycapnhat DESC NULLS LAST
            LIMIT 1
        ) latest ON true
        GROUP BY COALESCE(latest.tentrangthai, 'Chưa có trạng thái')
        ORDER BY value DESC, label
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let payment_methods = sqlx::query_as::<_, DashboardSlice>(
        r#"
        SELECT COALESCE(NULLIF(htthanhtoan, ''), 'Chưa xác định') AS label,
               COUNT(*)::bigint AS value
        FROM hoadon
        GROUP BY COALESCE(NULLIF(htthanhtoan, ''), 'Chưa xác định')
        ORDER BY value DESC, label
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let stock_summary = sqlx::query_as::<_, DashboardSlice>(
        r#"
        SELECT 'Còn hàng' AS label, COUNT(*)::bigint AS value
        FROM sanpham
        WHERE COALESCE(soluong, 0) > 10
        UNION ALL
        SELECT 'Sắp hết', COUNT(*)::bigint
        FROM sanpham
        WHERE COALESCE(soluong, 0) BETWEEN 1 AND 10
        UNION ALL
        SELECT 'Hết hàng', COUNT(*)::bigint
        FROM sanpham
        WHERE COALESCE(soluong, 0) = 0
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let recent_orders = sqlx::query_as::<_, OrderRow>(
        r#"
        SELECT TRIM(h.mahd) AS mahd, h.ngaylap, h.diachi, h.thanhtien, h.htthanhtoan,
               TRIM(h.mand) AS mand,
               nd.ten AS tenkh,
               TRIM(h.makm) AS makm,
               latest.tentrangthai AS trangthai,
               TRIM(latest.matt) AS matt,
               latest.ngaycapnhat
        FROM hoadon h
        LEFT JOIN nguoidung nd ON nd.mand = h.mand
        LEFT JOIN LATERAL (
            SELECT c.matt, t.tentrangthai, c.ngaycapnhat
            FROM cttrangthai c
            LEFT JOIN trangthai t ON t.matt = c.matt
            WHERE c.mahd = h.mahd
            ORDER BY c.ngaycapnhat DESC NULLS LAST
            LIMIT 1
        ) latest ON true
        ORDER BY h.ngaylap DESC NULLS LAST, h.mahd DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(DashboardVisuals {
        revenue,
        order_statuses,
        payment_methods,
        stock_summary,
        recent_orders,
    }))
}
