<style>
@import "@/assets/css/style.min.css";

.product-ratings .ratings {
    color: #FD5B5A !important;
}

.product-ratings .ratings::before {
    color: #FD5B5A !important;
}

/* 1. Ép khung bao ngoài ảnh chính về 468px (hoặc 100% cột) */
.product-single-carousel .product-item {
    max-width: 468px;
    /* Kích thước tối đa bạn muốn */
    margin: 0 auto;
    background: #fff;
}

/* 2. Ép ảnh chính hiển thị vuông và không tràn */
.product-single-image {
    width: 100% !important;
    height: auto !important;
    aspect-ratio: 1 / 1;
    /* Giữ tỉ lệ vuông */
    object-fit: contain;
    /* Dùng contain để thấy hết nhân vật anime, cover nếu muốn lấp đầy */
}

/* 3. Sửa lỗi Zoom Container: Ép nó phải đi theo độ rộng của cột cha */
.zoomContainer {
    max-width: 100% !important;
    height: auto !important;
}

.zoomWindowContainer {
    max-width: 100%;
}

/* 4. Ép Thumbnail về đúng 110px */
.prod-thumbnail .owl-dot {
    max-width: 110px;
    margin: 0 auto;
}

.prod-thumbnail .owl-dot img {
    width: 110px !important;
    height: 110px !important;
    object-fit: cover;
    border: 1px solid #e7e7e7;
    transition: border-color 0.3s;
}

/* Highlight ảnh đang chọn */
.prod-thumbnail .owl-dot.active img {
    border-color: #0088cc;
    /* Màu chủ đạo của shop */
}

.quantity-wrapper {
    display: flex;
    align-items: center;
    width: fit-content;
    border: 1px solid #ddd;
    overflow: hidden;
}

.qty-btn {
    width: 45px;
    height: 45px;
    border: none;
    background: white;
    font-size: 22px;
    cursor: pointer;
    transition: background 0.2s;
}

.qty-btn:hover {
    background: #f5f5f5;
}

.qty-input {
    width: 60px;
    height: 45px;
    border: none;
    border-left: 1px solid #ddd;
    border-right: 1px solid #ddd;
    text-align: center;
    font-size: 20px;
    font-weight: 600;
    outline: none;
}

/* Ẩn nút tăng giảm mặc định của input number */
.qty-input::-webkit-outer-spin-button,
.qty-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
}

.qty-input {
    appearance: none;
}

/* Fallback cho SPA: vẫn hiện ảnh chính nếu Owl Carousel chưa kịp khởi tạo sau khi API trả dữ liệu. */
.product-single-carousel:not(.owl-loaded) {
    display: block !important;
}

.product-single-carousel:not(.owl-loaded)>div:not(:first-child) {
    display: none;
}

.products-slider:not(.owl-loaded) {
    display: grid !important;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 2rem;
}

@media (max-width: 991px) {
    .products-slider:not(.owl-loaded) {
        grid-template-columns: repeat(3, minmax(0, 1fr));
    }
}

@media (max-width: 767px) {
    .products-slider:not(.owl-loaded) {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }
}

.product-action {
    display: flex !important;
    align-items: center !important;
    flex-wrap: wrap;
    gap: 10px;
}

.product-action>div,
.product-action .product-single-qty {
    display: flex;
    align-items: center;
    margin: 0 !important;
}

.product-action .quantity-wrapper,
.product-action .qty-btn,
.product-action .qty-input,
.product-action .add-cart,
.product-action .nolog {
    height: 48px !important;
    min-height: 48px !important;
    box-sizing: border-box !important;
}

.product-action .add-cart,
.product-action .nolog {
    display: inline-flex !important;
    align-items: center;
    justify-content: center;
    margin: 0 !important;
    padding: 0 2.5rem !important;
    line-height: 1 !important;
}

.product-action .qty-btn {
    font-size: 0;
}

.product-action .qty-btn::before {
    font-size: 1.7rem;
    line-height: 1;
}

.product-action .qty-btn:first-child::before {
    content: "−";
}

.product-action .qty-btn:last-child::before {
    content: "+";
}

.cart-message {
    display: flex;
    align-items: center;
    gap: 8px;
}

.cart-message::before {
    display: none !important;
}

.add-cart::before {
    display: none !important;
}

.add-cart::after {
    display: none !important;
}

.cart-message-content {
    display: flex;
    align-items: center;
    gap: 8px;
}

.cart-message-content.success i {
    color: #28a745;
}

.cart-message-content.error i {
    color: red;
}
</style>

<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from 'vue';
import axios from 'axios';
import { RouterLink, useRoute, useRouter } from 'vue-router';
import type { sanpham } from '@/models/sanpham';
import noUiSlider from 'nouislider';
import 'nouislider/dist/nouislider.css';
import { useGiohangStore } from '@/stores/XemnhanhGiohang'
import Danhgia from '@/components/Danhgia/Danhgia.vue'
import DetailSanphamLienquan from '@/components/Sanpham/DetailSanphamLienquan.vue'
import DetailSanphamNoibat from '@/components/Sanpham/DetailSanphamNoibat.vue'
import DetailSanphamMuanhieu from '@/components/Sanpham/DetailSanphamMuanhieu.vue'
import DetailSanphamDanhgiacao from '@/components/Sanpham/DetailSanphamDanhgiacao.vue'
import DetailSanphamMoiramat from '@/components/Sanpham/DetailSanphamMoiramat.vue'
import type { DetailProduct } from '@/models/detailProduct'
import { getProductImagePaths, handleProductImageError } from '@/utils/productImage'

const giohangStore = useGiohangStore()

interface SanPhamVoiChiTiet extends sanpham {
    tendmh?: string;
    tenhh?: string;
    diemtrungbinh?: number;
    hinhanhsps?: {
        maha?: string;
        duongdan?: string;
        masp?: string;
        anhdaidien?: number;
    }[];
}

const route = useRoute()
const router = useRouter();
const sp = ref<SanPhamVoiChiTiet | null>(null);
const tendmh = ref<string>('');
const lines = ref<string[]>([]);
const slug = route.params.slug as string
const masp = slug.split('_').pop()
const quantity = ref<number>(1)
const isLoggedIn = ref(false)
const isOverload = ref(false)
const reviewCount = ref(0)
const averageRating = ref(0)
const activeTab = ref<'description' | 'reviews'>('description')
const relatedSlider = ref<HTMLElement | null>(null)
const productGroups = ref({
    related: [] as DetailProduct[],
    featured: [] as DetailProduct[],
    bestSelling: [] as DetailProduct[],
    newest: [] as DetailProduct[],
    topRated: [] as DetailProduct[]
})

const initializeRelatedSlider = async (attempt = 0) => {
    await nextTick()
    const jquery = (window as any).jQuery
    if (!relatedSlider.value || !jquery?.fn?.owlCarousel) {
        if (attempt < 10) window.setTimeout(() => initializeRelatedSlider(attempt + 1), 150)
        return
    }

    const slider = jquery(relatedSlider.value)
    if (slider.hasClass('owl-loaded')) return
    slider.owlCarousel({
        items: 4,
        margin: 20,
        nav: true,
        dots: true,
        loop: productGroups.value.related.length > 4,
        responsive: {
            0: { items: 2 },
            768: { items: 3 },
            992: { items: 4 }
        }
    })
}

const loadProductGroups = async () => {
    if (!masp) return
    try {
        const response = await axios.get(`/api/detail-product-groups/${masp}`)
        productGroups.value = response.data
        await initializeRelatedSlider()
    } catch (error) {
        console.error('Không thể tải các nhóm sản phẩm:', error)
    }
}

const updateReviewSummary = (summary: { count: number; average: number }) => {
    reviewCount.value = summary.count
    averageRating.value = summary.average
}

const getAnhByLoai = (sp: SanPhamVoiChiTiet, loai: number) => {
    const images = getProductImagePaths(sp)
    return loai === 2 ? images.hover : images.main
};

const productImages = computed(() => {
    if (!sp.value) return []
    const images = getProductImagePaths(sp.value)
    return images.all.length ? images.all : [images.main]
})

const formatCurrency = (value: number | undefined) => {
    if (!value) return '0';
    return value.toLocaleString('vi-VN');
};

const increaseQty = () => {
    quantity.value++
}

const decreaseQty = () => {
    if (quantity.value > 1) {
        quantity.value--
    }
}

const checkOverload = () => {
    const stock = Number(sp.value?.soluong || 0)
    const remaining = masp ? giohangStore.getConlai(stock, masp) : stock
    isOverload.value = quantity.value > remaining
}

const showCartMessage = () => {
    const cartMessage = document.querySelector('.cart-message')
    cartMessage?.classList.remove('invisible')
    setTimeout(() => cartMessage?.classList.add('invisible'), 3000)
}

const loadSanphamChitiet = async () => {
    try {
        const res = await axios.get('/api/checkauth', {
            withCredentials: true
        })

        isLoggedIn.value = res.data.loggedIn
        if (isLoggedIn.value) void giohangStore.loadGiohang()

        const response = await axios.get('/api/loadSanphamChitiet', {
            params: {
                MaSp: masp
            }
        });

        // Khớp với cấu trúc Backend mới trả về { items, totalPages }
        sp.value = response.data.items[0] || null;
        tendmh.value = sp.value?.tendmh || 'Danh mục';
        lines.value = sp.value?.thongtin
            ?.split(/\r\n|\r|\n/)
            .filter(x => x.trim() !== '') || [];
        void loadProductGroups()
    } catch (error) {
        console.error("Lỗi:", error);
    }
};

const addGiohangLegacy = async () => {
    try {
        checkOverload();
        if (isOverload.value == false) {
            const res = await axios.post('/api/addGiohang',
                {
                    MaSp: masp,
                    Quanity: quantity.value
                },
                {
                    withCredentials: true
                }
            )
            console.log(res.data)

            // cập nhật realtime frontend
            giohangStore.addLocal({
                masp: sp.value?.masp,
                tensp: sp.value?.tensp,
                gia: sp.value?.gia,
                soluong: quantity.value,
                thanhtien: (sp.value?.gia || 0) * quantity.value,
                duongdan: getAnhByLoai(sp.value!, 1)
            })
        }
        // Hiện thông báo thêm thành công
        const cartMessage =
            document.querySelector('.cart-message')
        cartMessage?.classList.remove('invisible')
        setTimeout(() => {
            cartMessage?.classList.add('invisible')
        }, 3000)

    } catch (error) {
        console.error(error)
    }
}

const addGiohang = async () => {
    checkOverload()
    if (isOverload.value) {
        showCartMessage()
        return
    }

    try {
        await axios.post('/api/addGiohang', {
            MaSp: masp,
            Quanity: quantity.value
        }, { withCredentials: true })

        giohangStore.addLocal({
            masp: sp.value?.masp,
            tensp: sp.value?.tensp,
            gia: sp.value?.gia,
            soluong: quantity.value,
            thanhtien: (sp.value?.gia || 0) * quantity.value,
            duongdan: getAnhByLoai(sp.value!, 1)
        })
        isOverload.value = false
        showCartMessage()
    } catch (error: any) {
        if (error.response?.status === 409) {
            isOverload.value = true
            showCartMessage()
            return
        }
        console.error(error)
    }
}

onMounted(() => {
    loadSanphamChitiet()
})

watch(
    () => route.fullPath,
    () => {
        loadSanphamChitiet()
    }
)

// function selectStar(num) {
//     // Gán giá trị vào thẻ select ẩn
//     document.getElementById('Sao').value = num;

//     // (Tùy chọn) Thêm code CSS để highlight các ngôi sao đã chọn
//     const stars = document.querySelectorAll('.rating-stars a');
//     stars.forEach((star, index) => {
//         if (index < num) {
//             star.style.color = '#FD5B5A'; // Màu vàng cho sao được chọn
//         } else {
//             star.style.color = '#999'; // Màu xám cho sao còn lại
//         }
//     });
// }
</script>

<template>
    <main class="main">
        <div class="container">
            <nav aria-label="breadcrumb" class="breadcrumb-nav">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item">
                        <a :href="`/`"><i class="icon-home"></i>
                        </a>
                    </li>
                    <li class="breadcrumb-item"><a :href="`/Danhmuchang?MaDmh=${sp?.madmh}`">
                            {{ sp?.tendmh }}</a>
                    </li>
                    <li class="breadcrumb-item"><a :href="`/Hoathinh?MaHh=${sp?.mahh}`">
                            {{ sp?.tenhh }}</a>
                    </li>
                </ol>
            </nav>

            <div class="product-single-container product-single-default">
                <div class="cart-message invisible">
                    <template v-if="isOverload">

                        <div class="cart-message-content error">

                            <i class="icon-cancel"></i>

                            <strong>
                                Thêm quá số lượng trong kho. Vui lòng giảm xuống.
                            </strong>

                        </div>

                    </template>

                    <template v-else>

                        <div class="cart-message-content success">
                            <i class="icon-check"></i>
                            <strong class="single-cart-notice">
                                “{{ sp?.tensp }}”
                            </strong>

                            <span>
                                thêm vào giỏ hàng thành công.
                            </span>

                        </div>

                    </template>

                </div>

                <div class="row">
                    <div class="col-lg-5 col-md-6 product-single-gallery">
                        <div class="product-slider-container">
                            <div class="label-group">
                                <div class="product-label label-shot">HOT</div>
                            </div>

                            <div class="product-single-carousel owl-carousel owl-theme show-nav-hover">
                                <div v-for="(image, index) in productImages" :key="`${image}-${index}`">
                                    <div class="product-item">
                                        <img class="product-single-image" :src="image" :data-zoom-image="image"
                                            width="468" height="468" :alt="sp?.tensp || 'Sản phẩm'"
                                            @error="handleProductImageError" />
                                    </div>
                                </div>
                            </div>
                            <!-- End .product-single-carousel -->
                            <span class="prod-full-screen">
                                <i class="icon-plus"></i>
                            </span>
                        </div>

                        <div class="prod-thumbnail owl-dots">
                            <div v-for="(image, index) in productImages" :key="`thumb-${image}-${index}`">
                                <div class="owl-dot">
                                    <img class="product-single-image" :src="image" :data-zoom-image="image" width="110"
                                        height="110" :alt="`${sp?.tensp || 'Sản phẩm'} - ảnh ${index + 1}`"
                                        @error="handleProductImageError" />
                                </div>
                            </div>
                            <!-- @foreach (var image in Model.Hinhanhsps)
                            {
                            <div class="owl-dot">
                                <img src="@image.DuongDan" width="110" height="110" alt="product-thumbnail" />
                            </div>
                            } -->
                        </div>
                    </div>
                    <!-- End .product-single-gallery -->

                    <div class="col-lg-7 col-md-6 product-single-details">
                        <h1 class="product-title">{{ sp?.tensp }}</h1>

                        <div class="product-nav" style="display:none">
                            <div class="product-prev">
                                <a href="#">
                                    <span class="product-link"></span>

                                    <span class="product-popup">
                                        <span class="box-content">
                                            <img alt="product" width="150" height="150"
                                                :src="`../assets/images/products/product-3.jpg`"
                                                style="padding-top: 0px;">

                                            <span>Circled Ultimate 3D Speaker</span>
                                        </span>
                                    </span>
                                </a>
                            </div>

                            <div class="product-next">
                                <a href="#">
                                    <span class="product-link"></span>

                                    <span class="product-popup">
                                        <span class="box-content">
                                            <img alt="product" width="150" height="150"
                                                :src="`../assets/images/products/product-4.jpg`"
                                                style="padding-top: 0px;">

                                            <span>Blue Backpack for the Young</span>
                                        </span>
                                    </span>
                                </a>
                            </div>
                        </div>

                        <div class="ratings-container">
                            <div class="product-ratings">
                                <span class="ratings" :style="{ width: averageRating * 20 + '%' }"></span>

                                <span class="tooltiptext tooltip-top"></span>
                            </div>


                            <a href="#product-reviews-content" class="rating-link"
                                @click.prevent="activeTab = 'reviews'">
                                ( {{ reviewCount }} Đánh giá )
                            </a>
                        </div>

                        <hr class="short-divider">

                        <div class="price-box">
                            <span class="product-price"> {{ formatCurrency(sp?.gia) }} VND</span>
                        </div>
                        <!-- End .price-box -->

                        <div class="product-desc">
                            <p>
                                {{ sp?.ghichu }}
                            </p>
                        </div>
                        <!-- End .product-desc -->

                        <ul class="single-info-list">
                            <!---->
                            <li>
                                Mã sản phẩm:
                                <strong>{{ sp?.masp }}</strong>
                            </li>

                            <li>
                                Danh mục hàng:
                                <strong>
                                    <a :href="`/Danhmuchang?MaDmh=${sp?.madmh}`" class="product-category">{{ sp?.tendmh
                                    }}</a>
                                </strong>
                            </li>

                            <li>
                                Hoạt hình:
                                <strong>
                                    <a :href="`/Hoathinh?MaHh=${sp?.tenhh}`" class="product-category">{{ sp?.tenhh
                                    }}</a>
                                </strong>
                            </li>
                            <li>
                                Số lượng:
                                <strong>
                                    {{ sp?.soluong }}
                                </strong>
                            </li>
                            <!-- <li>
                                TAG:
                                @foreach(var tag in Model.MaTags)
                                {
                                <strong><a asp-action="Tag" asp-controller="SanphamsUser" asp-route-tags="@tag.MaTag"
                                        class="product-category">@tag.TenTag</a></strong>
                                if(tag != Model.MaTags.Last())
                                {
                                <span>, </span>
                                }
                                }
                            </li> -->
                        </ul>



                        <div class="product-action">


                            <div class="product-single-qty">
                                <div class="quantity-wrapper">
                                    <button class="qty-btn" @click="decreaseQty">
                                        −
                                    </button>

                                    <input v-model.number="quantity" type="number" min="1" class="qty-input" />

                                    <button class="qty-btn" @click="increaseQty">
                                        +
                                    </button>
                                </div>
                            </div>
                            <!-- End .product-single-qty -->
                            <div v-if="isLoggedIn">
                                <a href="javascript:;" class="btn btn-dark add-cart mr-2" @click="addGiohang">
                                    Thêm vào giỏ hàng
                                </a>
                            </div>
                            <div v-else>
                                <router-link to="/Dangnhap" class="btn btn-dark nolog mr-2" title="Thêm vào giỏ hàng">
                                    Thêm vào giỏ hàng
                                </router-link>
                            </div>

                        </div>
                        <!-- End .product-action -->
                    </div>
                    <!-- End .product-single-details -->
                </div>
                <!-- End .row -->
            </div>
            <!-- End .product-single-container -->

            <div class="product-single-tabs">
                <ul class="nav nav-tabs" role="tablist">
                    <li class="nav-item">
                        <a class="nav-link" :class="{ active: activeTab === 'description' }" id="product-tab-desc"
                            href="#product-desc-content" role="tab" @click.prevent="activeTab = 'description'">Thông tin
                            sản
                            phẩm</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" :class="{ active: activeTab === 'reviews' }" id="product-tab-reviews"
                            href="#product-reviews-content" role="tab" @click.prevent="activeTab = 'reviews'">
                            Đánh giá ({{ reviewCount }})
                        </a>
                    </li>
                </ul>

                <div class="tab-content">
                    <div v-show="activeTab === 'description'" class="tab-pane fade show active"
                        id="product-desc-content" role="tabpanel" aria-labelledby="product-tab-desc">
                        <div class="product-desc-content">
                            <p style="margin-bottom:15px;">
                                Chào mừng bạn đến với thế giới mua sắm đa dạng dành cho fan Anime, nơi mỗi sản phẩm
                                đều
                                được tuyển chọn kỹ lưỡng để mang lại trải nghiệm sưu tầm đẳng cấp nhất.
                                Dưới đây là chi tiết về thông số kỹ thuật và đặc điểm nổi bật của sản phẩm:
                            </p>
                            <ul>

                                <div v-for="line in lines">
                                    <li>
                                        {{ line.trim() }}
                                    </li>
                                </div>
                            </ul>
                        </div>
                        <!-- End .product-desc-content -->
                    </div>
                    <div v-show="activeTab === 'reviews'" class="tab-pane fade show active" id="product-reviews-content"
                        role="tabpanel" aria-labelledby="product-tab-reviews">
                        <div class="product-reviews-content">
                            <h3 class="reviews-title">Đánh giá cho {{ sp?.tensp }}</h3>

                            <div class="comment-list">
                                <Danhgia :masp="sp?.masp" @summary="updateReviewSummary" />
                            </div>

                            <div class="divider"></div>

                            <div v-if="false" class="add-product-review">
                                <form asp-action="Create" asp-controller="DanhgiaUser" method="post"
                                    class="comment-form m-0">
                                    <input type="hidden" name="MaSp" id="MaSp" value="@Model.MaSp" />
                                    <h3 class="review-title">Thêm đánh giá của bạn cho sản phẩm</h3>
                                    <div class="rating-form">
                                        <label for="rating">Xếp hạng <span class="required">*</span></label>
                                        <span class="rating-stars">
                                            <a class="star-1" href="javascript:void(0)" onclick="selectStar(1)">1</a>
                                            <a class="star-2" href="javascript:void(0)" onclick="selectStar(2)">2</a>
                                            <a class="star-3" href="javascript:void(0)" onclick="selectStar(3)">3</a>
                                            <a class="star-4" href="javascript:void(0)" onclick="selectStar(4)">4</a>
                                            <a class="star-5" href="javascript:void(0)" onclick="selectStar(5)">5</a>
                                        </span>

                                        <select name="Sao" id="Sao" required style="display: none;">
                                            <option value="">-- Chọn số sao --</option>
                                            <option value="5">Tuyệt vời</option>
                                            <option value="4">Tốt</option>
                                            <option value="3">Bình thường</option>
                                            <option value="2">Tệ</option>
                                            <option value="1">Rất tệ</option>
                                        </select>
                                    </div>

                                    <div class="form-group">
                                        <label>Nhận xét sản phẩm<span class="required">*</span></label>
                                        <textarea name="NoiDung" cols="5" rows="6" class="form-control form-control-sm"
                                            required></textarea>
                                    </div>
                                    <!-- @if (isUserLoggedIn)
                                    {
                                    <input type="submit" class="btn btn-primary" value="Gửi">
                                    }
                                    else
                                    {
                                    <a href="@Url.Action(" Index", "LoginUser" )" class="btn btn-primary">
                                        Gửi
                                    </a>
                                    } -->
                                </form>
                            </div>
                            <!-- End .add-product-review -->
                        </div>
                        <!-- End .product-reviews-content -->
                    </div>
                    <!-- End .tab-pane -->
                </div>
                <!-- End .tab-content -->
            </div>
            <!-- End .product-single-tabs -->

            <div class="products-section pt-0">
                <h2 class="section-title">Sản phẩm liên quan</h2>
                <div ref="relatedSlider" class="products-slider owl-carousel owl-theme dots-top dots-small">
                    <DetailSanphamLienquan :products="productGroups.related" />
                </div>
            </div>

            <hr class="mt-5 mb-5">

            <section class="product-widgets-container row pb-2">
                <div class="col-lg-3 col-sm-6 pb-5 pb-lg-0">
                    <h4 class="section-sub-title">Nổi bật nhất</h4>
                    <DetailSanphamNoibat :products="productGroups.featured" />
                </div>
                <div class="col-lg-3 col-sm-6 pb-5 pb-lg-0">
                    <h4 class="section-sub-title">Được mua nhiều</h4>
                    <DetailSanphamMuanhieu :products="productGroups.bestSelling" />
                </div>
                <div class="col-lg-3 col-sm-6 pb-5 pb-lg-0">
                    <h4 class="section-sub-title">Mới ra mắt</h4>
                    <DetailSanphamMoiramat :products="productGroups.newest" />
                </div>
                <div class="col-lg-3 col-sm-6 pb-5 pb-lg-0">
                    <h4 class="section-sub-title">Đánh giá cao</h4>
                    <DetailSanphamDanhgiacao :products="productGroups.topRated" />
                </div>
            </section>

            <div v-if="false" class="products-section pt-0">
                <h2 class="section-title">Sản phẩm liên quan</h2>
                <div class="products-slider owl-carousel owl-theme dots-top dots-small">
                    @await Component.InvokeAsync("Sanpham", new { view = "Related", MaHh = @Model.MaHh })
                </div>
                <!-- End .products-slider -->
            </div>
            <!-- End .products-section -->

            <hr class="mt-0 m-b-5" />



            <div v-if="false" class="product-widgets-container row pb-2">
                <div class="col-lg-3 col-sm-6 pb-5 pb-md-0">
                    <h4 class="section-sub-title">Nổi bật nhất</h4>
                    @await Component.InvokeAsync("Sanpham", new { view = "Featured_Detail" })
                </div>

                <div class="col-lg-3 col-sm-6 pb-5 pb-md-0">
                    <h4 class="section-sub-title">Được mua nhiều</h4>
                    @await Component.InvokeAsync("Chitietdonhang", "BestSales_Detail")
                </div>

                <div class="col-lg-3 col-sm-6 pb-5 pb-md-0">
                    <h4 class="section-sub-title">Mới ra mắt</h4>
                    @await Component.InvokeAsync("Sanpham", new { view = "NewArrivals_Detail" })
                </div>

                <div class="col-lg-3 col-sm-6 pb-5 pb-md-0">
                    <h4 class="section-sub-title">Đánh giá cao</h4>
                    @await Component.InvokeAsync("Danhgia", new { view = "TopRated_Detail" })
                </div>
            </div>
            <!-- End .row -->
        </div>
        <!-- End .container -->
    </main>
</template>
