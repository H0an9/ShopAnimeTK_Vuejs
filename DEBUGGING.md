# Debug bốn hệ thống trong VS Code

## Chuẩn bị một lần

1. Mở đúng thư mục gốc `ShopAnimeTK_Vuejs` bằng VS Code.
2. Cài các extension được VS Code đề xuất: Vue - Official, rust-analyzer và CodeLLDB.
3. Đảm bảo PostgreSQL và Ollama đang chạy nếu chức năng đang debug cần chúng.
4. Cài dependency frontend nếu chưa có:

```powershell
cd nodejs_Vue/frontend/ShopAnimeTK_nodejs
npm install
cd ../../../rust_Vue/frontend
npm install
```

## Bắt đầu debug

Mở **Run and Debug** (`Ctrl+Shift+D`), rồi chọn một cấu hình:

- `Debug: Node system`: Node backend và Vue frontend người dùng.
- `Debug: Rust system`: Rust backend và Vue frontend quản trị.
- `Debug: All 4 systems`: chạy đồng thời cả bốn hệ thống.
- Hoặc chọn riêng từng frontend/backend.

Các cổng cố định:

| Hệ thống | Cổng |
|---|---:|
| Node backend | 3000 |
| Node Vue frontend | 5173 |
| Rust backend | 8080 |
| Rust Vue frontend | 5174 |

Đặt breakpoint trực tiếp trong `.js`, `.ts`, `.vue` hoặc `.rs`, sau đó nhấn `F5`.

## Kiểm tra toàn bộ trước khi debug

Mở **Terminal → Run Task** và chạy `Check: All 4 systems`. Task này kiểm tra cú pháp Node backend, TypeScript frontend người dùng, biên dịch Rust và build frontend quản trị.

Nếu báo cổng đang được sử dụng, dừng tiến trình dev cũ trước khi nhấn `F5`. Các task Vite dùng `--strictPort` để không âm thầm chuyển sang cổng khác và làm sai proxy/API.
