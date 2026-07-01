const { pool } = require("./pool");

const getDangnhap = async ({taikhoan}) => {
    const result = await pool.query(
            `SELECT *,
            (
                SELECT COALESCE(JSON_AGG(anh), '[]'::json)
                FROM (
                    SELECT maha, duongdan, mand
                    FROM public.hinhanhnd
                    WHERE mand = nd.mand
                ) anh
            ) as hinhanhnd
            FROM public.nguoidung nd
            WHERE nd.taikhoan = $1
            AND phanquyen = B'0'`,
            [taikhoan]
        )
    return result;
}

const getAllDanhmuchang = async () => {
    const result = await pool.query('SELECT * FROM public.danhmuchang');
    return result.rows;
};

const getAllHoathinh = async () => {
    const result = await pool.query('SELECT * FROM public.hoathinh');
    return result.rows;
};

const getAllKhuyenmai = async () => {
    const result = await pool.query('SELECT * FROM public.khuyenmai');
    return result.rows;
};

const getSanphamNoibat = async () => {
    const result = await pool.query(`
        SELECT DISTINCT ON (TRIM(sp.madmh))
                sp.*,
                dmh.tendmh,
                COALESCE(dg.diemtrungbinh, 0) AS diemtrungbinh,
                COALESCE(dg.sodanhgia, 0)::integer AS sodanhgia,
                (
                    SELECT COALESCE(JSON_AGG(anh), '[]'::json)
                    FROM (
                        SELECT maha, duongdan, masp, anhdaidien
                        FROM public.hinhanhsp
                        WHERE TRIM(masp) = TRIM(sp.masp)
                          AND NULLIF(TRIM(duongdan), '') IS NOT NULL
                        ORDER BY
                            CASE WHEN anhdaidien = 1 THEN 0 WHEN anhdaidien = 2 THEN 1 ELSE 2 END,
                            maha
                    ) anh
                ) as hinhanhsps
        FROM public.sanpham sp
        LEFT JOIN public.danhmuchang dmh ON TRIM(dmh.madmh) = TRIM(sp.madmh)
        LEFT JOIN (
            SELECT TRIM(masp) AS masp, AVG(sao) AS diemtrungbinh, COUNT(*) AS sodanhgia
            FROM public.danhgia
            GROUP BY TRIM(masp)
        ) dg ON dg.masp = TRIM(sp.masp)
        WHERE TRIM(sp.madmh) NOT IN ('DMH002', 'DMH003')
        ORDER BY TRIM(sp.madmh), TRIM(sp.masp) ASC;
    `);
    return result.rows;
};

const getAllSanphamDmh = async ({ madmh, orderby, minPrice, maxPrice, page }) => {
    const pageSize = 12; // Số lượng sản phẩm mỗi trang
    const offset = (page - 1) * pageSize;

    // Xác định câu lệnh sắp xếp
    let sortQuery = "sp.masp DESC"; // Mặc định
    if (orderby === 'gia-asc') sortQuery = "sp.gia ASC";
    if (orderby === 'gia-desc') sortQuery = "sp.gia DESC";

    const queryText = `
        SELECT sp.*, dmh.tendmh,
            (
                SELECT COALESCE(JSON_AGG(anh), '[]'::json)
                FROM (
                    SELECT maha, duongdan, masp, anhdaidien 
                    FROM public.hinhanhsp 
                    WHERE TRIM(masp) = TRIM(sp.masp)
                      AND NULLIF(TRIM(duongdan), '') IS NOT NULL
                    ORDER BY
                        CASE WHEN anhdaidien = 1 THEN 0 WHEN anhdaidien = 2 THEN 1 ELSE 2 END,
                        maha
                ) anh
            ) as hinhanhsps,
            COUNT(*) OVER() as total_count -- Lấy tổng số dòng để tính totalPages
        FROM public.sanpham sp
        JOIN public.danhmuchang dmh ON TRIM(sp.madmh) = TRIM(dmh.madmh)
        WHERE TRIM(sp.madmh) ILIKE '%' || $1 || '%' 
          AND sp.gia >= $2 
          AND sp.gia <= $3
        ORDER BY ${sortQuery}
        LIMIT $4 OFFSET $5`;

    const values = [madmh, minPrice, maxPrice, pageSize, offset];
    
    const result = await pool.query(queryText, values);
    
    // Tính toán thông tin phân trang để trả về cho Vue
    const totalItems = result.rows.length > 0 ? parseInt(result.rows[0].total_count) : 0;
    const totalPages = Math.ceil(totalItems / pageSize);

    return {
        items: result.rows,
        totalPages: totalPages,
        totalItems: totalItems
    };
};

const getAllSanphamHh = async ({ mahh, orderby, minPrice, maxPrice, page }) => {
    const pageSize = 12; // Số lượng sản phẩm mỗi trang
    const offset = (page - 1) * pageSize;

    // Xác định câu lệnh sắp xếp
    let sortQuery = "sp.masp DESC"; // Mặc định
    if (orderby === 'gia-asc') sortQuery = "sp.gia ASC";
    if (orderby === 'gia-desc') sortQuery = "sp.gia DESC";

    const queryText = `
        SELECT sp.*, hh.tenhh,
            (
                SELECT COALESCE(JSON_AGG(anh), '[]'::json)
                FROM (
                    SELECT maha, duongdan, masp, anhdaidien 
                    FROM public.hinhanhsp 
                    WHERE TRIM(masp) = TRIM(sp.masp)
                      AND NULLIF(TRIM(duongdan), '') IS NOT NULL
                    ORDER BY
                        CASE WHEN anhdaidien = 1 THEN 0 WHEN anhdaidien = 2 THEN 1 ELSE 2 END,
                        maha
                ) anh
            ) as hinhanhsps,
            COUNT(*) OVER() as total_count -- Lấy tổng số dòng để tính totalPages
        FROM public.sanpham sp
        JOIN public.hoathinh hh ON TRIM(sp.mahh) = TRIM(hh.mahh)
        WHERE TRIM(sp.mahh) ILIKE '%' || $1 || '%'
          AND sp.gia >= $2 
          AND sp.gia <= $3
        ORDER BY ${sortQuery}
        LIMIT $4 OFFSET $5`;

    const values = [mahh, minPrice, maxPrice, pageSize, offset];
    
    const result = await pool.query(queryText, values);
    
    // Tính toán thông tin phân trang để trả về cho Vue
    const totalItems = result.rows.length > 0 ? parseInt(result.rows[0].total_count) : 0;
    const totalPages = Math.ceil(totalItems / pageSize);

    return {
        items: result.rows,
        totalPages: totalPages,
        totalItems: totalItems
    };
};

const getAllSanphamSearchHh = async ({ search, mahh, orderby, minPrice, maxPrice, page }) => {
    const pageSize = 12; // Số lượng sản phẩm mỗi trang
    const offset = (page - 1) * pageSize;

    // Xác định câu lệnh sắp xếp
    let sortQuery = "sp.masp DESC"; // Mặc định
    if (orderby === 'gia-asc') sortQuery = "sp.gia ASC";
    if (orderby === 'gia-desc') sortQuery = "sp.gia DESC";

    const queryText = `
        SELECT sp.*, hh.tenhh,
            (
                SELECT COALESCE(JSON_AGG(anh), '[]'::json)
                FROM (
                    SELECT maha, duongdan, masp, anhdaidien 
                    FROM public.hinhanhsp 
                    WHERE TRIM(masp) = TRIM(sp.masp)
                      AND NULLIF(TRIM(duongdan), '') IS NOT NULL
                    ORDER BY
                        CASE WHEN anhdaidien = 1 THEN 0 WHEN anhdaidien = 2 THEN 1 ELSE 2 END,
                        maha
                ) anh
            ) as hinhanhsps,
            COUNT(*) OVER() as total_count -- Lấy tổng số dòng để tính totalPages
        FROM public.sanpham sp
        LEFT JOIN public.hoathinh hh ON TRIM(sp.mahh) = TRIM(hh.mahh)
        WHERE ($1::text IS NULL OR sp.tensp ILIKE '%' || $1::text || '%')
          AND ($2::text IS NULL OR TRIM(sp.mahh) ILIKE '%' || $2::text || '%')
          AND sp.gia >= $3 
          AND sp.gia <= $4
        ORDER BY ${sortQuery}
        LIMIT $5 OFFSET $6`;

    const values = [search, mahh, minPrice, maxPrice, pageSize, offset];
    
    const result = await pool.query(queryText, values);
    
    // Tính toán thông tin phân trang để trả về cho Vue
    const totalItems = result.rows.length > 0 ? parseInt(result.rows[0].total_count) : 0;
    const totalPages = Math.ceil(totalItems / pageSize);

    return {
        items: result.rows,
        totalPages: totalPages,
        totalItems: totalItems
    };
};

const getAllSanphamChitiet = async ({ masp }) => {
    const queryText = `
        SELECT sp.*, dmh.tendmh, hh.tenhh,
            (
                SELECT JSON_AGG(anh) 
                FROM (
                    SELECT maha, duongdan, masp, anhdaidien 
                    FROM public.hinhanhsp 
                    WHERE TRIM(masp) = TRIM(sp.masp)
                      AND NULLIF(TRIM(duongdan), '') IS NOT NULL
                    ORDER BY
                        CASE WHEN anhdaidien = 1 THEN 0 WHEN anhdaidien = 2 THEN 1 ELSE 2 END,
                        maha
                ) anh
            ) as hinhanhsps,
            COUNT(*) OVER() as total_count -- Lấy tổng số dòng để tính totalPages
        FROM public.sanpham sp
        JOIN public.danhmuchang dmh ON TRIM(sp.madmh) = TRIM(dmh.madmh)
        JOIN public.hoathinh hh ON TRIM(sp.mahh) = TRIM(hh.mahh)
        WHERE TRIM(sp.masp) = TRIM($1) `;

    const values = [masp];
    const result = await pool.query(queryText, values);
    return {
        items: result.rows
    };
};

const updateStudents = async ({ id, name, age, hometown }) => {
    try {
        const result = await pool.query(
            'UPDATE public.students SET name=$1, age=$2, hometown=$3 WHERE id=$4',
            [name, age, hometown, id]);

        return "success";
    } catch (error) {
        return "error";
    }
};

const insertStudents = async ({ name, age, hometown }) => {
    try {
        const result = await pool.query(
            'INSERT INTO public.students (name, age, hometown) VALUES ($1, $2, $3)',
            [name, age, hometown]
        );

        return "success";
    } catch (error) {
        return "error";
    }
};

const deleteStudents = async ({ id }) => {
    try {
        const result = await pool.query(
            'DELETE FROM public.students WHERE id=$1',
            [id]
        );

        return "success";
    } catch (error) {
        return "error";
    }
}

const getPagination = async ({ page, text }) => {
    const limit = 10;
    const offset = (page - 1) * limit;

    let result, countRes;

    if (text) {
        result = await pool.query(
            'SELECT * FROM students WHERE name ILIKE $1 ORDER BY id LIMIT $2 OFFSET $3',
            [`%${text}%`, limit, offset]
        );

        countRes = await pool.query(
            'SELECT COUNT(*) FROM students WHERE name ILIKE $1',
            [`%${text}%`]
        );
    } else {
        result = await pool.query(
            'SELECT * FROM students ORDER BY id LIMIT $1 OFFSET $2',
            [limit, offset]
        );

        countRes = await pool.query('SELECT COUNT(*) FROM students');
    }

    const totalItems = parseInt(countRes.rows[0].count);
    const totalPages = Math.ceil(totalItems / limit);

    return {
        students: result.rows,
        total: totalItems,
        page,
        totalPages
    };
};

const getStudentsByID = async ({ id }) => {
    try {
        const result = await pool.query(
            'SELECT * FROM students WHERE id = $1',
            [id]);

        return result.rows[0];
    } catch (error) {
        return "error";
    }
}

module.exports = { getAllDanhmuchang, getAllHoathinh, getSanphamNoibat, getAllSanphamDmh, getAllSanphamHh, getAllSanphamSearchHh, 
                    getAllSanphamChitiet, getDangnhap, getPagination, updateStudents, insertStudents, deleteStudents, getStudentsByID,
                    getAllKhuyenmai }
