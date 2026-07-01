use super::{ApiError, ApiResult};
use crate::{
    email::send_best_effort,
    models::{
        AdminSession, ChangePasswordInput, ForgotPasswordInput, LoginInput, LoginResponse,
        UpdateProfileInput,
    },
    AppState,
};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/forgot-password", post(forgot_password))
        .route("/me", get(me))
        .route("/profile", axum::routing::put(update_profile))
        .route("/password", axum::routing::put(change_password))
        .route("/logout", post(logout))
}
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|v| !v.is_empty())
}
async fn session_user(state: &AppState, headers: &HeaderMap) -> ApiResult<(String, AdminSession)> {
    let token = bearer_token(headers)
        .ok_or_else(|| ApiError::unauthorized("Phiên đăng nhập không hợp lệ"))?;
    let json=sqlx::query_scalar::<_,String>("UPDATE user_sessions SET expire=NOW()+INTERVAL '7 days' WHERE sid=$1 AND expire>NOW() RETURNING sess::text")
        .bind(token).fetch_optional(&state.pool).await?.ok_or_else(||ApiError::unauthorized("Phiên đăng nhập đã hết hạn"))?;
    let user = serde_json::from_str(&json)
        .map_err(|_| ApiError::unauthorized("Phiên đăng nhập không hợp lệ"))?;
    Ok((token.to_string(), user))
}
async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> ApiResult<Json<LoginResponse>> {
    let username = input.taikhoan.trim();
    if username.is_empty() || input.matkhau.is_empty() {
        return Err(ApiError::bad_request("Vui lòng nhập tài khoản và mật khẩu"));
    }
    let row=sqlx::query_as::<_,(String,Option<String>,String,Option<String>)>("SELECT TRIM(mand),ten,taikhoan,email FROM nguoidung WHERE taikhoan=$1 AND matkhau=$2 AND phanquyen=B'1' AND trangthai=B'1' LIMIT 1")
        .bind(username).bind(&input.matkhau).fetch_optional(&state.pool).await?;
    let (mand, ten, taikhoan, email) = row.ok_or_else(|| {
        ApiError::unauthorized("Tài khoản hoặc mật khẩu không đúng, hoặc bạn không có quyền admin")
    })?;
    let user = AdminSession {
        mand,
        ten,
        taikhoan,
        email,
    };
    let token = Uuid::new_v4().to_string();
    let json = serde_json::to_string(&user)
        .map_err(|_| ApiError::internal("Không thể tạo phiên đăng nhập"))?;
    sqlx::query("DELETE FROM user_sessions WHERE expire<=NOW()")
        .execute(&state.pool)
        .await?;
    sqlx::query(
        "INSERT INTO user_sessions(sid,sess,expire) VALUES($1,$2::json,NOW()+INTERVAL '7 days')",
    )
    .bind(&token)
    .bind(json)
    .execute(&state.pool)
    .await?;
    Ok(Json(LoginResponse { token, user }))
}
async fn forgot_password(
    State(state): State<AppState>,
    Json(input): Json<ForgotPasswordInput>,
) -> ApiResult<Json<serde_json::Value>> {
    let email = input.email.trim();
    if email.is_empty() {
        return Err(ApiError::bad_request("Vui long nhap email admin"));
    }
    let new_password = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(10)
        .collect::<String>();
    let admin = sqlx::query_as::<_, (String, Option<String>, String, String)>(
        r#"
        UPDATE nguoidung
        SET matkhau=$2
        WHERE LOWER(TRIM(email))=LOWER(TRIM($1))
          AND phanquyen=B'1'
          AND trangthai=B'1'
        RETURNING TRIM(mand), ten, taikhoan, email
        "#,
    )
    .bind(email)
    .bind(&new_password)
    .fetch_optional(&state.pool)
    .await?;
    if let Some((_mand, name, username, admin_email)) = admin {
        let subject = "Mat khau admin Shop Anime TK moi";
        let body=format!(
            "Xin chao {},\n\nMat khau admin cua ban da duoc tao moi.\n\nTai khoan: {}\nMat khau moi: {}\n\nVui long dang nhap va doi lai mat khau ngay sau do.",
            name.as_deref().unwrap_or("admin"),username,new_password
        );
        send_best_effort(Some(admin_email.as_str()), subject, &body).await;
    }
    Ok(Json(
        serde_json::json!({"message":"Neu email ton tai va co quyen admin, mat khau moi se duoc gui den email do."}),
    ))
}
async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<AdminSession>> {
    let (_, user) = session_user(&state, &headers).await?;
    Ok(Json(user))
}
async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<UpdateProfileInput>,
) -> ApiResult<Json<AdminSession>> {
    let (token, current) = session_user(&state, &headers).await?;
    if input.ten.trim().is_empty() {
        return Err(ApiError::bad_request("Tên hiển thị không được để trống"));
    }
    let email = input
        .email
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    sqlx::query("UPDATE nguoidung SET ten=$2,email=$3 WHERE TRIM(mand)=TRIM($1)")
        .bind(&current.mand)
        .bind(input.ten.trim())
        .bind(&email)
        .execute(&state.pool)
        .await?;
    let updated = AdminSession {
        ten: Some(input.ten.trim().to_string()),
        email,
        ..current
    };
    let json = serde_json::to_string(&updated)
        .map_err(|_| ApiError::internal("Không thể cập nhật phiên"))?;
    sqlx::query(
        "UPDATE user_sessions SET sess=$2::json,expire=NOW()+INTERVAL '7 days' WHERE sid=$1",
    )
    .bind(token)
    .bind(json)
    .execute(&state.pool)
    .await?;
    Ok(Json(updated))
}
async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ChangePasswordInput>,
) -> ApiResult<Json<serde_json::Value>> {
    let (_, current) = session_user(&state, &headers).await?;
    if input.matkhau_moi.len() < 6 {
        return Err(ApiError::bad_request(
            "Mật khẩu mới phải có ít nhất 6 ký tự",
        ));
    }
    let result =
        sqlx::query("UPDATE nguoidung SET matkhau=$3 WHERE TRIM(mand)=TRIM($1) AND matkhau=$2")
            .bind(&current.mand)
            .bind(&input.matkhau_hientai)
            .bind(&input.matkhau_moi)
            .execute(&state.pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::bad_request("Mật khẩu hiện tại không đúng"));
    }
    Ok(Json(
        serde_json::json!({"message":"Đổi mật khẩu thành công"}),
    ))
}
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(token) = bearer_token(&headers) {
        sqlx::query("DELETE FROM user_sessions WHERE sid=$1")
            .bind(token)
            .execute(&state.pool)
            .await?;
    }
    Ok(Json(serde_json::json!({"message":"Đã đăng xuất"})))
}
pub async fn require_admin(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> ApiResult<Response> {
    let path = request.uri().path();
    if path.ends_with("/auth/login") || path.ends_with("/auth/forgot-password") {
        return Ok(next.run(request).await);
    }
    let token = bearer_token(request.headers())
        .ok_or_else(|| ApiError::unauthorized("Vui lòng đăng nhập bằng tài khoản admin"))?;
    let result = sqlx::query(
        "UPDATE user_sessions SET expire=NOW()+INTERVAL '7 days' WHERE sid=$1 AND expire>NOW()",
    )
    .bind(token)
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::unauthorized("Phiên đăng nhập đã hết hạn"));
    }
    Ok(next.run(request).await)
}
