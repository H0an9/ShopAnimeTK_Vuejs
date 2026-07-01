use super::{ApiError, ApiResult};
use crate::{
    db::new_id,
    email::send_best_effort,
    models::{PageResponse, UserInput, UserRow},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::AssertSqlSafe;

#[derive(Deserialize)]
struct UserListQuery {
    page: Option<i64>,
    page_size: Option<i64>,
    q: Option<String>,
    role: Option<String>,
    active: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", put(update).delete(remove))
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> ApiResult<Json<PageResponse<UserRow>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(5, 100);
    let q = query.q.unwrap_or_default();
    let role = query.role.unwrap_or_default();
    let active = query.active.unwrap_or_default();
    let conditions = "WHERE ($1='' OR mand ILIKE '%'||$1||'%' OR ten ILIKE '%'||$1||'%' OR taikhoan ILIKE '%'||$1||'%' OR email ILIKE '%'||$1||'%') AND ($2='' OR phanquyen = CASE WHEN $2='1' THEN B'1' ELSE B'0' END) AND ($3='' OR ($3='1' AND trangthai = B'1') OR ($3='0' AND COALESCE(trangthai, B'0') = B'0'))";
    let count_sql = format!("SELECT COUNT(*) FROM nguoidung {conditions}");
    let total = sqlx::query_scalar::<_, i64>(AssertSqlSafe(count_sql))
        .bind(&q)
        .bind(&role)
        .bind(&active)
        .fetch_one(&state.pool)
        .await?;
    let list_sql = format!(
        r#"
        SELECT mand, ten, ngaysinh, taikhoan,
               trangthai::text AS trangthai,
               phanquyen::text AS phanquyen,
               email, thoigian
        FROM nguoidung
        {conditions}
        ORDER BY mand DESC
        LIMIT $4 OFFSET $5
        "#
    );
    let rows = sqlx::query_as::<_, UserRow>(AssertSqlSafe(list_sql))
        .bind(&q)
        .bind(&role)
        .bind(&active)
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(PageResponse::new(rows, total, page, page_size)))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<UserInput>,
) -> ApiResult<(StatusCode, Json<UserRow>)> {
    let id = input.mand.unwrap_or_else(|| new_id("U"));
    let pass = input.matkhau.unwrap_or_else(|| "123456".to_string());
    let initial_password = pass.clone();
    let active = input.trangthai.unwrap_or_else(|| "1".to_string());
    let role = input.phanquyen.unwrap_or_else(|| "0".to_string());

    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO nguoidung (mand, ten, ngaysinh, taikhoan, matkhau, trangthai, phanquyen, email, thoigian)
        VALUES ($1, $2, $3, $4, $5,
                CASE WHEN $6 = '1' THEN B'1' ELSE B'0' END,
                CASE WHEN $7 = '1' THEN B'1' ELSE B'0' END,
                $8, NOW())
        RETURNING mand, ten, ngaysinh, taikhoan, trangthai::text AS trangthai, phanquyen::text AS phanquyen, email, thoigian
        "#,
    )
    .bind(id).bind(input.ten).bind(input.ngaysinh).bind(input.taikhoan).bind(pass)
    .bind(active).bind(role).bind(input.email)
    .fetch_one(&state.pool).await?;

    let subject = "Tai khoan Shop Anime TK da duoc tao";
    let body = format!(
        "Xin chao {},\n\nTai khoan cua ban tren he thong Shop Anime TK da duoc tao.\n\nTai khoan: {}\nMat khau: {}\nQuyen: {}\n\nVui long dang nhap va doi mat khau sau khi nhan duoc email nay.",
        row.ten.as_deref().unwrap_or("ban"),
        row.taikhoan.as_deref().unwrap_or("-"),
        initial_password,
        if row.phanquyen.as_deref() == Some("1") { "Admin" } else { "User" }
    );
    send_best_effort(row.email.as_deref(), subject, &body).await;

    Ok((StatusCode::CREATED, Json(row)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UserInput>,
) -> ApiResult<Json<UserRow>> {
    let active = input.trangthai.unwrap_or_else(|| "1".to_string());
    let role = input.phanquyen.unwrap_or_else(|| "0".to_string());

    if let Some(pass) = input.matkhau {
        sqlx::query(
            r#"
            UPDATE nguoidung
            SET ten=$2, ngaysinh=$3, taikhoan=$4, matkhau=$5,
                trangthai=CASE WHEN $6='1' THEN B'1' ELSE B'0' END,
                phanquyen=CASE WHEN $7='1' THEN B'1' ELSE B'0' END,
                email=$8
            WHERE TRIM(mand)=TRIM($1)
            "#,
        )
        .bind(&id)
        .bind(&input.ten)
        .bind(input.ngaysinh)
        .bind(&input.taikhoan)
        .bind(pass)
        .bind(&active)
        .bind(&role)
        .bind(&input.email)
        .execute(&state.pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE nguoidung
            SET ten=$2, ngaysinh=$3, taikhoan=$4,
                trangthai=CASE WHEN $5='1' THEN B'1' ELSE B'0' END,
                phanquyen=CASE WHEN $6='1' THEN B'1' ELSE B'0' END,
                email=$7
            WHERE TRIM(mand)=TRIM($1)
            "#,
        )
        .bind(&id)
        .bind(&input.ten)
        .bind(input.ngaysinh)
        .bind(&input.taikhoan)
        .bind(&active)
        .bind(&role)
        .bind(&input.email)
        .execute(&state.pool)
        .await?;
    }

    let row = sqlx::query_as::<_, UserRow>(
        r#"SELECT TRIM(mand) AS mand, ten, ngaysinh, taikhoan, trangthai::text AS trangthai, phanquyen::text AS phanquyen, email, thoigian
           FROM nguoidung WHERE TRIM(mand)=TRIM($1)"#,
    ).bind(&id).fetch_optional(&state.pool).await?
        .ok_or_else(|| ApiError::not_found("Không tìm thấy người dùng"))?;

    Ok(Json(row))
}

async fn remove(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM nguoidung WHERE mand = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
