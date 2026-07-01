use super::{ApiError, ApiResult};
use crate::{
    db::new_id,
    models::{PageResponse, Product, ProductImage, ProductInput},
    AppState,
};
use axum::{
    body::Bytes,
    extract::Query,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use sqlx::AssertSqlSafe;
use std::{
    path::{Path as FilePath, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(get_one).put(update).delete(remove))
        .route("/:id/images", get(list_images).post(upload_image))
        .route("/:id/images/:image_id", axum::routing::delete(delete_image))
        .route(
            "/:id/images/:image_id/primary",
            axum::routing::put(set_primary_image),
        )
}

#[derive(Deserialize)]
struct UploadImageQuery {
    filename: String,
}

#[derive(Deserialize)]
struct ProductListQuery {
    page: Option<i64>,
    page_size: Option<i64>,
    q: Option<String>,
    category: Option<String>,
    anime: Option<String>,
    stock: Option<String>,
    sort: Option<String>,
}

fn image_root() -> PathBuf {
    std::env::var("PRODUCT_IMAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../frontend/public/img/products"))
}

fn user_image_root() -> Option<PathBuf> {
    std::env::var("USER_PRODUCT_IMAGE_ROOT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            Some(PathBuf::from(
                "../../nodejs_Vue/frontend/ShopAnimeTK_nodejs/public/img/products",
            ))
        })
}

fn safe_segment(value: &str, fallback: &str) -> String {
    let clean: String = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .collect();
    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean
    }
}

fn image_extension(filename: &str, headers: &HeaderMap) -> ApiResult<&'static str> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let extension = FilePath::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match (content_type, extension.as_str()) {
        ("image/jpeg", _) | (_, "jpg" | "jpeg") => Ok("jpg"),
        ("image/png", _) | (_, "png") => Ok("png"),
        ("image/webp", _) | (_, "webp") => Ok("webp"),
        ("image/gif", _) | (_, "gif") => Ok("gif"),
        _ => Err(ApiError::bad_request(
            "Chỉ hỗ trợ ảnh JPG, PNG, WEBP hoặc GIF",
        )),
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn product_select() -> &'static str {
    r#"
    SELECT
        s.masp, s.tensp, s.gia, s.ghichu, s.madmh, s.mahh, s.thongtin, s.soluong,
        d.tendmh,
        h.tenhh,
        img.duongdan AS anhdaidien
    FROM sanpham s
    LEFT JOIN danhmuchang d ON TRIM(d.madmh) = TRIM(s.madmh)
    LEFT JOIN hoathinh h ON TRIM(h.mahh) = TRIM(s.mahh)
    LEFT JOIN LATERAL (
        SELECT duongdan
        FROM hinhanhsp
        WHERE TRIM(masp) = TRIM(s.masp)
          AND NULLIF(TRIM(duongdan), '') IS NOT NULL
        ORDER BY
            CASE WHEN anhdaidien = 1 THEN 0 ELSE 1 END,
            anhdaidien DESC NULLS LAST,
            maha
        LIMIT 1
    ) img ON true
    "#
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<ProductListQuery>,
) -> ApiResult<Json<PageResponse<Product>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(5, 100);
    let q = query.q.unwrap_or_default();
    let category = query.category.unwrap_or_default();
    let anime = query.anime.unwrap_or_default();
    let stock = query.stock.unwrap_or_default();
    let order = match query.sort.as_deref() {
        Some("name") => "s.tensp ASC NULLS LAST",
        Some("price_asc") => "s.gia ASC NULLS LAST",
        Some("price_desc") => "s.gia DESC NULLS LAST",
        Some("stock") => "s.soluong ASC NULLS LAST",
        _ => "s.masp DESC",
    };
    let conditions = r#"
      WHERE ($1 = '' OR s.masp ILIKE '%' || $1 || '%' OR s.tensp ILIKE '%' || $1 || '%')
        AND ($2 = '' OR TRIM(s.madmh) = $2)
        AND ($3 = '' OR TRIM(s.mahh) = $3)
        AND ($4 = '' OR ($4 = 'out' AND COALESCE(s.soluong,0)=0)
          OR ($4 = 'low' AND COALESCE(s.soluong,0) BETWEEN 1 AND 10)
          OR ($4 = 'available' AND COALESCE(s.soluong,0)>0))
    "#;
    let count_sql = format!("SELECT COUNT(*) FROM sanpham s {conditions}");
    let total = sqlx::query_scalar::<_, i64>(AssertSqlSafe(count_sql))
        .bind(&q)
        .bind(&category)
        .bind(&anime)
        .bind(&stock)
        .fetch_one(&state.pool)
        .await?;
    let sql = format!(
        "{} {} ORDER BY {} LIMIT $5 OFFSET $6",
        product_select(),
        conditions,
        order
    );
    let items = sqlx::query_as::<_, Product>(AssertSqlSafe(sql))
        .bind(&q)
        .bind(&category)
        .bind(&anime)
        .bind(&stock)
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(PageResponse::new(items, total, page, page_size)))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Product>> {
    let sql = format!("{} WHERE TRIM(s.masp) = TRIM($1)", product_select());
    let item = sqlx::query_as::<_, Product>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(item))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<ProductInput>,
) -> ApiResult<(StatusCode, Json<Product>)> {
    let id = input.masp.unwrap_or_else(|| new_id("SP"));
    sqlx::query(
        r#"
        INSERT INTO sanpham (masp, tensp, gia, ghichu, madmh, mahh, thongtin, soluong)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&id)
    .bind(&input.tensp)
    .bind(input.gia)
    .bind(&input.ghichu)
    .bind(&input.madmh)
    .bind(&input.mahh)
    .bind(&input.thongtin)
    .bind(input.soluong)
    .execute(&state.pool)
    .await?;

    let sql = format!("{} WHERE TRIM(s.masp) = TRIM($1)", product_select());
    let item = sqlx::query_as::<_, Product>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ProductInput>,
) -> ApiResult<Json<Product>> {
    sqlx::query(
        r#"
        UPDATE sanpham
        SET tensp = $2, gia = $3, ghichu = $4, madmh = $5, mahh = $6, thongtin = $7, soluong = $8
        WHERE masp = $1
        "#,
    )
    .bind(&id)
    .bind(&input.tensp)
    .bind(input.gia)
    .bind(&input.ghichu)
    .bind(&input.madmh)
    .bind(&input.mahh)
    .bind(&input.thongtin)
    .bind(input.soluong)
    .execute(&state.pool)
    .await?;

    let sql = format!("{} WHERE TRIM(s.masp) = TRIM($1)", product_select());
    let item = sqlx::query_as::<_, Product>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(item))
}

async fn remove(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    // Xoá ảnh/tag/chi tiết liên quan trước để tránh dữ liệu mồ côi.
    sqlx::query("DELETE FROM hinhanhsp WHERE masp = $1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    sqlx::query("DELETE FROM chitiettag WHERE masp = $1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    sqlx::query("DELETE FROM sanpham WHERE masp = $1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_images(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Vec<ProductImage>>> {
    let images = sqlx::query_as::<_, ProductImage>(
        r#"
        SELECT maha, duongdan, masp, anhdaidien
        FROM hinhanhsp
        WHERE TRIM(masp) = TRIM($1)
        ORDER BY CASE WHEN anhdaidien = 1 THEN 0 ELSE 1 END, maha
        "#,
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(images))
}

async fn upload_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<UploadImageQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<ProductImage>)> {
    if body.is_empty() {
        return Err(ApiError::bad_request("File ảnh đang trống"));
    }
    if body.len() > 10 * 1024 * 1024 {
        return Err(ApiError::bad_request("Mỗi ảnh không được vượt quá 10 MB"));
    }

    let category: Option<String> =
        sqlx::query_scalar("SELECT madmh FROM sanpham WHERE TRIM(masp) = TRIM($1)")
            .bind(&id)
            .fetch_one(&state.pool)
            .await?;
    let category = safe_segment(category.as_deref().unwrap_or("KHAC").trim(), "KHAC");
    let extension = image_extension(&query.filename, &headers)?;
    let original_stem = FilePath::new(&query.filename)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("product");
    let stem = safe_segment(original_stem, "product");
    let suffix = unique_suffix();
    let filename = format!("{}_{}.{}", stem, suffix, extension);
    let directory = image_root().join(&category);
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| ApiError::internal(format!("Không tạo được thư mục ảnh: {error}")))?;
    tokio::fs::write(directory.join(&filename), &body)
        .await
        .map_err(|error| ApiError::internal(format!("Không lưu được ảnh: {error}")))?;

    if let Some(user_root) = user_image_root() {
        let user_directory = user_root.join(&category);
        tokio::fs::create_dir_all(&user_directory)
            .await
            .map_err(|error| {
                ApiError::internal(format!("Khong tao duoc thu muc anh user: {error}"))
            })?;
        tokio::fs::write(user_directory.join(&filename), &body)
            .await
            .map_err(|error| ApiError::internal(format!("Khong luu duoc anh user: {error}")))?;
    }

    let has_primary: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM hinhanhsp WHERE TRIM(masp) = TRIM($1) AND anhdaidien = 1)",
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;
    let image_id = format!("HA{:08}", suffix % 100_000_000);
    let public_path = format!("/img/products/{category}/{filename}");
    let primary = if has_primary { 0 } else { 1 };
    let image = sqlx::query_as::<_, ProductImage>(
        r#"
        INSERT INTO hinhanhsp (maha, duongdan, masp, anhdaidien)
        VALUES ($1, $2, $3, $4)
        RETURNING maha, duongdan, masp, anhdaidien
        "#,
    )
    .bind(image_id)
    .bind(public_path)
    .bind(id)
    .bind(primary)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(image)))
}

async fn set_primary_image(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let mut transaction = state.pool.begin().await?;
    sqlx::query("UPDATE hinhanhsp SET anhdaidien = 0 WHERE TRIM(masp) = TRIM($1)")
        .bind(&id)
        .execute(&mut *transaction)
        .await?;
    let result = sqlx::query(
        "UPDATE hinhanhsp SET anhdaidien = 1 WHERE TRIM(masp) = TRIM($1) AND maha = $2",
    )
    .bind(&id)
    .bind(&image_id)
    .execute(&mut *transaction)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Không tìm thấy ảnh sản phẩm"));
    }
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_image(
    State(state): State<AppState>,
    Path((id, image_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let image: ProductImage = sqlx::query_as(
        "SELECT maha, duongdan, masp, anhdaidien FROM hinhanhsp WHERE TRIM(masp) = TRIM($1) AND maha = $2",
    )
    .bind(&id)
    .bind(&image_id)
    .fetch_one(&state.pool)
    .await?;
    sqlx::query("DELETE FROM hinhanhsp WHERE TRIM(masp) = TRIM($1) AND maha = $2")
        .bind(&id)
        .bind(&image_id)
        .execute(&state.pool)
        .await?;

    if let Some(public_path) = image.duongdan {
        let relative = public_path.trim_start_matches("/img/products/");
        if !relative.contains("..") {
            let _ = tokio::fs::remove_file(image_root().join(relative)).await;
            if let Some(user_root) = user_image_root() {
                let _ = tokio::fs::remove_file(user_root.join(relative)).await;
            }
        }
    }
    if image.anhdaidien == Some(1) {
        sqlx::query(
            r#"
            UPDATE hinhanhsp SET anhdaidien = 1
            WHERE maha = (
                SELECT maha FROM hinhanhsp WHERE TRIM(masp) = TRIM($1) ORDER BY maha LIMIT 1
            )
            "#,
        )
        .bind(id)
        .execute(&state.pool)
        .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
