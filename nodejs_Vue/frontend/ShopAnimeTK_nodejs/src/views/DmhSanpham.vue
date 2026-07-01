<style>
@import "@/assets/css/style.min.css";

.product-ratings .ratings {
    color: #FD5B5A !important;
}

.product-ratings .ratings::before {
    color: #FD5B5A !important;
}

li.active a {
    color: #08c !important;
    font-weight: bold;
}

.my-product-image {
    display: block;
    width: 100% !important;
    max-width: 280px;
    aspect-ratio: 1 / 1;
    overflow: hidden;
    position: relative;
    background: #fff;
}

.my-product-image img {
    width: 100% !important;
    height: 100% !important;
    object-fit: contain;
    display: block;
}

.price-slider-wrapper {
    padding: 10px 0;
    width: 100%;
}

#price-slider {
    width: 100%;
    margin-bottom: 15px;
}

.filter-price-action {
    margin-top: 10px;
}

.filter-price-text {
    font-size: 14px;
}
</style>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import axios from 'axios';
import { RouterLink, useRoute, useRouter } from 'vue-router';
import type { sanpham } from '@/models/sanpham';
import { getProductImagePaths, handleProductImageError } from '@/utils/productImage'
import type { danhmuchang } from '@/models/danhmuchang';
import noUiSlider from 'nouislider';
import 'nouislider/dist/nouislider.css';

interface SanPhamVoiChiTiet extends sanpham {
    tendmh?: string;
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
const sps = ref<SanPhamVoiChiTiet[]>([]);
const madmh = computed(() => route.query.MaDmh as string || '')
const tendmh = ref<string>('');
const priceSlider = ref<HTMLElement | null>(null);
let requestId = 0

const getAnhByLoai = (sp: SanPhamVoiChiTiet, loai: number) => {
    const images = getProductImagePaths(sp)
    return loai === 2 ? images.hover : images.main
};

const formatCurrency = (value: number | undefined) => {
    if (!value) return '0';
    return value.toLocaleString('vi-VN');
};

const totalPages = ref(1); // Cần khai báo để template sử dụng

// State quản lý bộ lọc - Ép kiểu Number để tính toán chính xác
const filters = ref({
    madmh: (route.query.MaDmh as string) ?? '',
    orderby: (route.query.orderby as string) ?? 'menu_order',
    minPrice: Number(route.query.minPrice) || 0,
    maxPrice: Number(route.query.maxPrice) || 10000000,
    page: Number(route.query.page) || 1
});

const syncFiltersFromRoute = () => {
    filters.value.madmh = String(route.query.MaDmh || '').trim();
    filters.value.orderby = (route.query.orderby as string) ?? 'menu_order';
    filters.value.minPrice = Number(route.query.minPrice) || 0;
    filters.value.maxPrice = Number(route.query.maxPrice) || 10000000;
    filters.value.page = Number(route.query.page) || 1;
}

const loadSanphamDmh = async () => {
    const currentRequestId = ++requestId
    const params = { ...filters.value }
    try {
        const response = await axios.get('/api/loadSanphamDmh', {
            // Gửi trực tiếp MaDmh viết hoa theo yêu cầu Backend nếu cần, 
            // hoặc dùng filters.value.madmh
            params: {
                MaDmh: params.madmh,
                orderby: params.orderby,
                minPrice: params.minPrice,
                maxPrice: params.maxPrice,
                page: params.page
            }
        });
        if (currentRequestId !== requestId) return

        // Khớp với cấu trúc Backend mới trả về { items, totalPages }
        sps.value = response.data.items || [];
        totalPages.value = response.data.totalPages || 1;

        if (sps.value.length > 0) {
            tendmh.value = sps.value[0]?.tendmh || 'Danh mục';
        }
    } catch (error) {
        console.error("Lỗi:", error);
    }
};

const applyFilter = () => {
    router.push({
        query: {
            MaDmh: filters.value.madmh,
            orderby: filters.value.orderby,
            minPrice: filters.value.minPrice,
            maxPrice: filters.value.maxPrice,
            page: filters.value.page
        }
    });
};

onMounted(() => {
    if (!priceSlider.value) return;

    noUiSlider.create(priceSlider.value, {
        start: [filters.value.minPrice, filters.value.maxPrice],
        connect: true,
        range: {
            min: 0,
            max: 10000000
        },
        step: 10000,
        format: {
            to: (value: number) => Math.round(value).toString(),
            from: (value: string) => Number(value)
        }
    });

    const sliderInstance = (priceSlider.value as any).noUiSlider;

    sliderInstance.on(
        'update',
        (values: string[]) => {
            filters.value.minPrice = Number(values[0]);
            filters.value.maxPrice = Number(values[1]);
        }
    );
});

watch(
    () => route.fullPath,
    () => {
        syncFiltersFromRoute();

        const sliderInstance = (priceSlider.value as any)?.noUiSlider;

        if (sliderInstance) {
            sliderInstance.set([
                filters.value.minPrice,
                filters.value.maxPrice
            ]);
        }

        loadSanphamDmh();
    },
    { immediate: true }
);
</script>

<template>
    <main class="main">
        <div class="category-banner-container bg-gray">
            <div class="category-banner banner text-uppercase"
                style="background: no-repeat 60%/cover url('assets/images/banners/banner-top.jpg');">
                <div class="container position-relative">
                    <div class="row">
                        <div class="pl-lg-5 pb-5 pb-md-0 col-sm-5 col-xl-4 col-lg-4 offset-1">
                            <h3>Electronic<br>Deals</h3>
                            <a href="category.html" class="btn btn-dark">Get Yours!</a>
                        </div>
                        <div class="pl-lg-3 col-sm-4 offset-sm-0 offset-1 pt-3">
                            <div class="coupon-sale-content">
                                <h4 class="m-b-1 coupon-sale-text bg-white text-transform-none">
                                    Exclusive COUPON
                                </h4>
                                <h5 class="mb-2 coupon-sale-text d-block ls-10 p-0"><i class="ls-0">UP TO</i><b
                                        class="text-dark">$100</b> OFF</h5>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="container">
            <nav aria-label="breadcrumb" class="breadcrumb-nav">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item">
                        <router-link :to="`/`">
                            <i class="icon-home"></i>
                        </router-link>
                    </li>
                    <li class="breadcrumb-item">
                        <router-link :to="`/Danhmuchang?MaDmh=${madmh}`">
                            {{ tendmh }}
                        </router-link>
                    </li>
                </ol>
            </nav>

            <div class="row">
                <div class="col-lg-9 main-content">
                    <nav class="toolbox sticky-header" data-sticky-options="{'mobile': true}" style="height:50px">
                        <div class="toolbox-left">

                            <!--Sắp xếp-->
                            <div class="toolbox-item toolbox-sort">
                                <label>Sắp xếp:</label>
                                <div class="select-custom">
                                    <select v-model="filters.orderby" class="form-control" @change="applyFilter">
                                        <option value="menu_order">Mặc định</option>
                                        <option value="gia-desc">Giá: Cao đến thấp</option>
                                        <option value="gia-asc">Giá: Thấp đến cao</option>
                                    </select>
                                </div>
                            </div>

                        </div>
                        <!-- End .toolbox-left -->


                    </nav>
                    <div v-if="sps && sps.length > 0">

                        <div class="row">
                            <div v-for="sp in sps" :key="sp.masp" class="col-6 col-sm-4">

                                <div class="product-default">
                                    <figure>
                                        <router-link
                                            :to="`/chitiet/${sp.tensp?.toLowerCase().replace(/\s+/g, '-')}_${sp.masp}`"
                                            :title="sp.tensp" class="my-product-image">
                                            <img :src="getAnhByLoai(sp, 1)" :alt="sp.tensp || 'Sản phẩm'" @error="handleProductImageError" />
                                            <img :src="getAnhByLoai(sp, 2)" :alt="`${sp.tensp || 'Sản phẩm'} - ảnh phụ`" class="hover-image" @error="handleProductImageError" />
                                        </router-link>
                                        <!--Nhãn giảm giá-->
                                        <div class="label-group">
                                            <div class="product-label label-hot">HOT</div>
                                        </div>
                                    </figure>
                                    <!--Danh mục hàng-->
                                    <div class="product-details">
                                        <div class="category-wrap">
                                            <div class="category-list">
                                                <router-link :to="`/Danhmuchang?MaDmh=${sp.madmh}`"
                                                    class="product-category">
                                                    {{ sp.tendmh }}
                                                </router-link>
                                            </div>
                                        </div>

                                        <h3 class="product-title">
                                            <router-link :to="`/chitiet/${sp.tensp?.toLowerCase().replace(/\s+/g, '-')}_${sp.masp}`"
                                                :title="`${sp.tensp}`">
                                                {{ sp.tensp }}
                                            </router-link>
                                        </h3>

                                        <div class="ratings-container">
                                            <div class="product-ratings">
                                                <span class="ratings"
                                                    :style="{ width: (sp.diemtrungbinh || 0) * 20 + '%' }"></span>
                                                <!-- End .ratings -->
                                                <span class="tooltiptext tooltip-top"></span>
                                            </div>
                                            <!-- End .product-ratings -->
                                        </div>
                                        <!-- End .product-container -->

                                        <div class="price-box">
                                            <span class="product-price">{{ formatCurrency(sp.gia) }} đ</span>
                                        </div>
                                        <!-- End .price-box -->

                                        <div class="product-action">
                                            <a href="wishlist.html" class="btn-icon-wish" title="Yêu thích">
                                                <i class="icon-heart"></i>
                                            </a>
                                            <router-link :to="`/chitiet/${sp.tensp?.toLowerCase().replace(/\s+/g, '-')}_${sp.masp}`"
                                                class="btn-icon btn-add-cart">
                                                <i class="fa fa-arrow-right"></i><span>
                                                    MUA HÀNG
                                                </span>
                                            </router-link>
                                            <!--<a href="ajax/product-quick-view.html" class="btn-quickview" title="Quick View"><i class="fas fa-external-link-alt"></i></a>-->
                                        </div>
                                    </div>
                                    <!-- End .product-details -->
                                </div>

                            </div>



                        </div>
                        <!--Phân trang-->
                        <nav class="toolbox toolbox-pagination">
                            <ul class="pagination toolbox-item">
                                <!-- Nút lùi -->
                                <li class="page-item" :class="{ disabled: filters.page <= 1 }">
                                    <a class="page-link" @click="filters.page--; applyFilter()">
                                        <i class="icon-angle-left"></i>
                                    </a>
                                </li>

                                <!-- Các số trang (Giả sử có biến totalPages từ API) -->
                                <li v-for="p in totalPages" :key="p" class="page-item"
                                    :class="{ active: p === filters.page }">
                                    <a class="page-link" @click="filters.page = p; applyFilter()">{{ p }}</a>
                                </li>

                                <!-- Nút tới -->
                                <li class="page-item" :class="{ disabled: filters.page >= totalPages }">
                                    <a class="page-link" @click="filters.page++; applyFilter()">
                                        <i class="icon-angle-right"></i>
                                    </a>
                                </li>
                            </ul>
                        </nav>
                        <!-- <nav class="toolbox toolbox-pagination">
                                <ul class="pagination toolbox-item">
                                    @Html.PagedListPager(Model,
                                    page => Url.Action("Danhmuchang", new { MaDmh = ViewBag.MaDmh, MaHh = ViewBag.MaHh,
                                    orderby = ViewBag.Sort, minPrice = ViewBag.SelectedMin, maxPrice =
                                    ViewBag.SelectedMax,
                                    page }),
                                    new PagedListRenderOptions
                                    {
                                    UlElementClasses = new[] { "pagination", "toolbox-item" },
                                    LiElementClasses = new[] { "page-item" },
                                    PageClasses = new[] { "page-link" },

                                    LinkToPreviousPageFormat = "<i class='icon-angle-left'></i>",
                                    LinkToNextPageFormat = "<i class='icon-angle-right'></i>",

                                    DisplayLinkToFirstPage = PagedListDisplayMode.Always,
                                    DisplayLinkToLastPage = PagedListDisplayMode.Always,

                                    DisplayLinkToPreviousPage = PagedListDisplayMode.Always,
                                    DisplayLinkToNextPage = PagedListDisplayMode.Always,

                                    DisplayEllipsesWhenNotShowingAllPageNumbers = true,
                                    EllipsesElementClass = "page-item",
                                    MaximumPageNumbersToDisplay = 5
                                    }
                                    )
                                </ul>
                            </nav> -->
                    </div>
                    <div v-else>
                        <div class="row text-center">
                            <div class="col-12">
                                Không tìm thấy sản phẩm nào phù hợp.
                            </div>
                        </div>
                    </div>
                </div>
                <!-- End .col-lg-9 -->

                <div class="sidebar-overlay"></div>
                <aside class="sidebar-shop col-lg-3 order-lg-first mobile-sidebar">
                    <div class="sidebar-wrapper">
                        <!-- End .widget -->

                        <div class="widget">
                            <h3 class="widget-title">
                                <a data-toggle="collapse" href="#widget-body-3" role="button" aria-expanded="true"
                                    aria-controls="widget-body-3">Giá thành</a>
                            </h3>

                            <div class="collapse show" id="widget-body-3">
                                <div class="widget-body pb-0">
                                    <!--Lọc theo giá-->
                                    <form @submit.prevent="applyFilter">
                                        <div class="price-slider-wrapper">
                                            <div ref="priceSlider" id="price-slider"></div>
                                        </div>

                                        <div
                                            class="filter-price-action d-flex align-items-center justify-content-between flex-wrap">
                                            <div class="filter-price-text">
                                                Giá:
                                                {{ formatCurrency(filters.minPrice) }}đ -
                                                {{ formatCurrency(filters.maxPrice) }}đ
                                            </div>

                                            <button type="submit" class="btn btn-primary">
                                                Lọc
                                            </button>
                                        </div>
                                    </form>
                                </div>
                                <!-- End .widget-body -->
                            </div>
                        </div>



                        <div class="widget widget-block">
                            <h3 class="widget-title">Chính sách giao hàng</h3>
                            <h5>Vận chuyển siêu tốc</h5>
                            <p>Hỗ trợ giao hàng hỏa tốc trong nội thành. Kiểm tra hàng thoải mái trước khi thanh
                                toán (Ship COD). </p>
                        </div>
                        <!-- End .widget -->
                    </div>
                    <!-- End .sidebar-wrapper -->
                </aside>
                <!-- End .col-lg-3 -->
            </div>
            <!-- End .row -->
        </div>
        <!-- End .container -->

        <div class="mb-4"></div>
        <!-- margin -->
    </main>
    <!-- End .main -->
</template>
