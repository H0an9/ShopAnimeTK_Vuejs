<script setup lang="ts">
import TheWelcome from '../components/TheWelcome.vue'
import DanhmuchangSidebar from '@/components/Danhmuchang/DanhmuchangSidebar.vue';
import SanphamNoibat from '@/components/Sanpham/SanphamNoibat.vue';
import Muanhieu from '@/components/Sanpham/Muanhieu.vue'
import Danhgiacao from '@/components/Sanpham/Danhgiacao.vue'
import Moiramat from '@/components/Sanpham/Moiramat.vue'
import type { DetailProduct } from '@/models/detailProduct'
import { onMounted, ref } from 'vue'
import axios from 'axios'

const homeProductGroups = ref({
  bestSelling: [] as DetailProduct[],
  topRated: [] as DetailProduct[],
  newest: [] as DetailProduct[]
})

onMounted(async () => {
  try {
    const response = await axios.get('/api/home-product-groups')
    homeProductGroups.value = response.data
  } catch (error) {
    console.error('Không thể tải các nhóm sản phẩm trang chủ:', error)
  }
})
</script>

<style>
/* Ép menu bọc ngoài không cho phép các mục con nhảy hàng đôi */
.side-nav .menu-vertical {
  display: flex !important;
  flex-direction: column !important;
  padding: 0;
  margin: 0;
  list-style: none;
  width: 100% !important;
}

/* Đảm bảo mỗi danh mục là một hàng độc lập chiếm 100% chiều rộng */
.side-nav .menu-vertical>li {
  display: block !important;
  width: 100% !important;
  float: none !important;
  /* Phá vỡ thuộc tính float nếu có */
  clear: both !important;
  border-bottom: 1px solid #e7e7e7;
  /* Làm mượt đường gạch ngang */
}

/* Định dạng thẻ chứa chữ và icon thẳng hàng */
.side-nav .menu-vertical>li>a {
  display: flex !important;
  align-items: center;
  padding: 12px 15px !important;
  width: 100% !important;
  box-sizing: border-box;
  text-decoration: none;
  color: #333;
}

/* Khoảng cách giữa Icon và Chữ */
.side-nav .menu-vertical>li>a>i {
  margin-right: 15px !important;
  width: 20px;
  text-align: center;
}

/* Hover menu bên trái */
.side-nav .menu-vertical>li>a:hover {
  background-color: #08c !important;
  /* màu nền hover */
  color: #fff !important;
  /* màu chữ */
}

/* Icon cũng đổi màu theo */
.side-nav .menu-vertical>li>a:hover i {
  color: #fff !important;
}

/* Nếu có trạng thái active */
.side-nav .menu-vertical>li.active>a {
  background-color: #08c !important;
  color: #fff !important;
}

.side-nav .menu-vertical>li.active>a i {
  color: #fff !important;
}

.product-ratings .ratings {
  color: #FD5B5A !important;
}

.product-ratings .ratings::before {
  color: #FD5B5A !important;
}

.my-product-image {
  display: block;
  width: 160px !important;
  height: 160px !important;
  overflow: hidden;
  position: relative;
}

.my-product-image img {
  width: 100% !important;
  height: 100% !important;
  object-fit: cover;
  display: block;
}

.product-default .newArr {
  width: 84px !important;
  height: 84px !important;
  object-fit: cover;
  /* Giữ tỷ lệ ảnh và cắt vừa khung */
  display: block;
}

.brand-logo {
  max-width: 140px;
  /* không vượt quá 140px */
  max-height: 60px;
  /* không vượt quá 60px */
  width: auto;
  height: auto;
  display: block;
  margin: 0 auto;
}
</style>


<template>
  <main>
    <main class="main home">
      <div class="container mb-2">
        <div class="info-boxes-container row row-joined mb-2 font2">
          <div class="info-box info-box-icon-left col-lg-4">
            <i class="icon-shipping"></i>

            <div class="info-box-content">
              <h4>MIỄN PHÍ GIAO HÀNG &amp; ĐỔI TRẢ</h4>
              <p class="text-body">Freeship đơn trong Hà Nội.</p>
            </div>
            <!-- End .info-box-content -->
          </div>
          <!-- End .info-box -->

          <div class="info-box info-box-icon-left col-lg-4">
            <i class="icon-money"></i>

            <div class="info-box-content">
              <h4>CAM KẾT HOÀN TIỀN</h4>
              <p class="text-body">Giá trị hoàn tiền lên đến 100%.</p>
            </div>
            <!-- End .info-box-content -->
          </div>
          <!-- End .info-box -->

          <div class="info-box info-box-icon-left col-lg-4">
            <i class="icon-support"></i>

            <div class="info-box-content">
              <h4>HỖ TRỢ TRỰC TUYẾN 24/7</h4>
              <p class="text-body">Đội ngũ nhân viên chuyên nghiệp.</p>
            </div>
            <!-- End .info-box-content -->
          </div>
          <!-- End .info-box -->
        </div>

        <div class="row">
          <div class="col-lg-9">
            <div class="home-slider slide-animate owl-carousel owl-theme mb-2" data-owl-options="{
       'loop': false,
       'dots': true,
       'nav': false
      }">
              <div class="home-slide home-slide1 banner banner-md-vw banner-sm-vw d-flex align-items-center">
                <img class="slide-bg" style="background-color: #2699D0;"
                  src="@/assets/images/demoes/demo1/slider/slide-4.png" width="880" height="428" alt="home-slider">
                <div class="banner-layer appear-animate" data-animation-name="fadeInUpShorter">
                  <h4 class="text-white mb-0">Tìm ra giới hạn. Vượt qua!</h4>
                  <h2 class="text-white mb-1">Sales Mùa Đông</h2>
                  <h3 class="text-white text-uppercase m-b-3">Giá Tốt</h3>
                  <h5 class="text-white text-uppercase d-inline-block mb-0 ls-n-20 align-text-bottom">
                    Chỉ từ <b class="coupon-sale-text bg-secondary text-white d-inline-block">
                      <em class="align-text-top">199</em>999 VND
                    </b>
                  </h5>
                  <router-link to="/Hoathinh?MaHh=Hh010" class="btn btn-dark btn-md ls-10">Mua ngay!</router-link>
                </div>
                <!-- End .banner-layer -->
              </div>
              <!-- End .home-slide -->

              <div class="home-slide home-slide2 banner banner-md-vw banner-sm-vw d-flex align-items-center">
                <img class="slide-bg" style="background-color: #dadada;"
                  src="@/assets/images/demoes/demo1/slider/slide-5.jpg" width="880" height="428" alt="home-slider">
                <div class="banner-layer text-uppercase appear-animate" data-animation-name="fadeInUpShorter">
                  <h4 class="m-b-2">Chuyến phiêu lưu vô tận</h4>
                  <h2 class="m-b-3">Deal sốc</h2>
                  <h5 class="d-inline-block mb-0 align-top mr-5 mb-2">
                    Chỉ từ
                    <b><em>299</em>999 VND</b>
                  </h5>
                  <router-link to="/Hoathinh?MaHh=Hh009" class="btn btn-dark btn-md ls-10">Sở hữu ngay!</router-link>
                </div>
                <!-- End .banner-layer -->
              </div>
              <!-- End .home-slide -->

              <div class="home-slide home-slide3 banner banner-md-vw banner-sm-vw  d-flex align-items-center">
                <img class="slide-bg" style="background-color: #e5e4e2;"
                  src="@/assets/images/demoes/demo1/slider/slide-6.jpg" width="880" height="428" alt="home-slider" />
                <div class="banner-layer text-uppercase appear-animate" data-animation-name="fadeInUpShorter">
                  <h4 class="m-b-2">Ưu đãi độc quyền</h4>
                  <h2 class="m-b-3">Hàng mới về</h2>
                  <h5 class="d-inline-block mb-0 align-top mr-5 mb-2">
                    Chỉ từ
                    <b><em>99</em>99 VND</b>
                  </h5>
                  <router-link to="/Hoathinh?MaHh=Hh001" class="btn btn-dark btn-md ls-10">Săn ngay!</router-link>
                </div>
                <!-- End .banner-layer -->
              </div>
              <!-- End .home-slide -->
            </div>
            <!-- End .home-slider -->

            <div class="banners-container m-b-2 owl-carousel owl-theme" data-owl-options="{
       'dots': false,
       'margin': 20,
       'loop': false,
       'responsive': {
        '480': {
         'items': 2
        },
        '768': {
         'items': 3
        }
       }
      }">
              <div class="banner banner1 banner-hover-shadow d-flex align-items-center mb-2 w-100 appear-animate"
                data-animation-name="fadeInLeftShorter" data-animation-delay="500">
                <figure class="w-100">
                  <img src="@/assets/images/demoes/demo1/banners/banner-4.png" style="background-color: #dadada;"
                    alt="banner">
                </figure>
                <div class="banner-layer">
                  <h3 class="m-b-2">Phong cách mới</h3>
                  <h4 class="m-b-4 text-primary"><sup class="text-dark"><del>Chìm</del></sup> Trình<sup></sup></h4>
                  <router-link :to="{ name: 'Danhmuchang', query: { MaDmh: 'DMH007' } }"
                    class="text-dark text-uppercase ls-10">
                    Mua ngay
                  </router-link>
                </div>
              </div>
              <!-- End .banner -->
              <div
                class="banner banner2 text-uppercase banner-hover-shadow d-flex align-items-center mb-2 w-100 appear-animate"
                data-animation-name="fadeInUpShorter" data-animation-delay="200">
                <figure class="w-100">
                  <img src="@/assets/images/demoes/demo1/banners/banner-5.png" style="background-color: #dadada;"
                    alt="banner">
                </figure>
                <div class="banner-layer text-center">
                  <h3 class="m-b-1 ls-n-20">Siêu Deal</h3>
                  <h4 class="text-body">Chỉ từ 99.999</h4>
                  <router-link :to="`/Hoathinh?MaHh=Hh002`" class="text-dark text-uppercase ls-10">
                    Mua ngay
                  </router-link>
                </div>
              </div>
              <!-- End .banner -->
              <div class="banner banner3 banner-hover-shadow d-flex align-items-center mb-2 w-100 appear-animate"
                data-animation-name="fadeInRightShorter" data-animation-delay="500">
                <figure class="w-100">
                  <img src="@/assets/images/demoes/demo1/banners/banner-6.png" style="background-color: #dadada;"
                    alt="banner">
                </figure>
                <div class="banner-layer text-right">
                  <h3 class="m-b-2">Mô hình</h3>
                  <h4 class="mb-3 text-secondary text-uppercase">Chỉ từ 650K</h4>
                  <router-link :to="`/Danhmuchang?MaDmh=DMH001`" class="text-dark text-uppercase ls-10">
                    Mua ngay
                  </router-link>
                </div>
              </div>
              <!-- End .banner -->
            </div>

            <h2 class="section-title ls-n-10 m-b-4 appear-animate" data-animation-name="fadeInUpShorter">
              Sản phẩm nổi bật
            </h2>

            <div class="products-slider owl-carousel owl-theme dots-top dots-small m-b-1 pb-1 appear-animate"
              data-animation-name="fadeInUpShorter">
              <SanphamNoibat>

              </SanphamNoibat>
            </div>


            <!-- End .featured-proucts -->

            <div class="brands-slider owl-carousel owl-theme images-center appear-animate" data-animation-name="fadeIn"
              data-animation-duration="700"
              data-owl-options="{
                            'margin': 0,'responsive': {'768': {'items': 4}, '991': {'items': 4},'1200': {'items': 5}}}">
              <!-- @await Component.InvokeAsync("Hoathinh", new { view = "LogoImg" }) -->
            </div>
            <!-- End .brands-slider -->

            <div class="row products-widgets">
              <div class="col-sm-6 col-md-4 pb-4 pb-md-0 appear-animate" data-animation-name="fadeInLeftShorter"
                data-animation-delay="200">
                <div class="product-column">
                  <h3 class="section-sub-title ls-n-20">Đánh giá cao</h3>
                  <Danhgiacao :products="homeProductGroups.topRated" />
                  <!-- End .product-column -->
                </div>
              </div>
              <!-- End .col-md-4 -->

              <div class="col-sm-6 col-md-4 pb-4 pb-md-0 appear-animate" data-animation-name="fadeInLeftShorter"
                data-animation-delay="500">
                <div class="product-column">
                  <h3 class="section-sub-title ls-n-20">Bán chạy nhất</h3>
                  <Muanhieu :products="homeProductGroups.bestSelling" />
                </div>
                <!-- End .product-column -->
              </div>
              <!-- End .col-md-4 -->

              <div class="col-sm-6 col-md-4 pb-4 pb-md-0 appear-animate" data-animation-name="fadeInLeftShorter"
                data-animation-delay="800">
                <div class="product-column">
                  <h3 class="section-sub-title ls-n-20">Hàng mới về</h3>
                  <Moiramat :products="homeProductGroups.newest" />
                </div>
                <!-- End .product-column -->
              </div>
              <!-- End .col-md-4 -->
            </div>
            <!-- End .row -->

            <hr class="mt-1 mb-3 pb-2">

            <div class="feature-boxes-container">
              <div class="row">
                <div class="col-md-4 appear-animate" data-animation-name="fadeInRightShorter"
                  data-animation-delay="200">
                  <div class="feature-box  feature-box-simple text-center">
                    <i class="icon-earphones-alt"></i>

                    <div class="feature-box-content p-0">
                      <h3 class="mb-0 pb-1">Hỗ trợ khách hàng</h3>
                      <h5 class="mb-1 pb-1">Cần trợ giúp?</h5>

                      <p>Đội ngũ hỗ trợ khách hàng của chúng tôi luôn sẵn sàng giải đáp mọi thắc mắc của bạn ngay lập
                        tức!.</p>
                    </div>
                    <!-- End .feature-box-content -->
                  </div>
                  <!-- End .feature-box -->
                </div>
                <!-- End .col-md-4 -->

                <div class="col-md-4 appear-animate" data-animation-name="fadeInRightShorter"
                  data-animation-delay="400">
                  <div class="feature-box feature-box-simple text-center">
                    <i class="icon-credit-card"></i>

                    <div class="feature-box-content p-0">
                      <h3 class="mb-0 pb-1">Thanh toán linh hoạt</h3>
                      <h5 class="mb-1 pb-1">An toàn & Nhanh chóng</h5>

                      <p>Thanh toán minh bạch, không phí ẩn – hoàn toàn an tâm, nhận hàng trong vài bước.</p>
                    </div>
                    <!-- End .feature-box-content -->
                  </div>
                  <!-- End .feature-box -->
                </div>
                <!-- End .col-md-4 -->

                <div class="col-md-4 appear-animate" data-animation-name="fadeInRightShorter"
                  data-animation-delay="600">
                  <div class="feature-box feature-box-simple text-center">
                    <i class="icon-action-undo"></i>

                    <div class="feature-box-content p-0">
                      <h3 class="mb-0 pb-1">Đổi trả</h3>
                      <h5 class="mb-1 pb-1">Dễ dàng & Miễn phí</h5>

                      <p>Đổi trả với quy trình đơn giản, trải nghiệm mua sắm không lo rủi ro!.</p>
                    </div>
                    <!-- End .feature-box-content -->
                  </div>
                  <!-- End .feature-box -->
                </div>
                <!-- End .col-md-4 -->
              </div>
              <!-- End .row -->
            </div>
            <!-- End .feature-boxes-container -->
          </div>
          <!-- End .col-lg-9 -->

          <div class="sidebar-overlay"></div>
          <div class="sidebar-toggle custom-sidebar-toggle"><i class="fas fa-sliders-h"></i></div>
          <aside class="sidebar-home col-lg-3 order-lg-first mobile-sidebar">

            <div class="side-menu-wrapper text-uppercase mb-2 d-none d-lg-block">
              <h2 class="side-menu-title bg-gray ls-n-25">Danh mục hàng</h2>

              <nav class="side-nav">
                <ul class="menu menu-vertical sf-arrows">
                  <DanhmuchangSidebar></DanhmuchangSidebar>
                </ul>
              </nav>
            </div>
            <!-- End .side-menu-container -->

          </aside>
          <!-- End .col-lg-3 -->
        </div>
        <!-- End .row -->
      </div>
      <!-- End .container -->
    </main>
    <!-- End .main -->
  </main>
</template>
