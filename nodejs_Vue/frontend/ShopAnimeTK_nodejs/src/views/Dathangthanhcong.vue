<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import axios from 'axios'
import type { khuyenmai } from '@/models/khuyenmai'

interface OrderDetail {
    masp: string
    tensp?: string
    gia: number
    soluong: number
}

interface OrderSuccess {
    mahd: string
    ngaylap: string
    diachi: string
    thanhtien: number
    htthanhtoan: string
    makm?: string
    details: OrderDetail[]
    promotion: khuyenmai | null
    subtotal: number
    discount: number
    shippingFee: number
}

const route = useRoute()
const order = ref<OrderSuccess>()
const isLoading = ref(true)
const errorMessage = ref('')

const addressParts = computed(() => order.value?.diachi?.split('_') ?? [])
const receiverName = computed(() => addressParts.value[0] || '')
const receiverPhone = computed(() => addressParts.value[1] || '')
const receiverProvince = computed(() => addressParts.value[2] || '')
const receiverAddress = computed(() => addressParts.value.slice(3).join('_'))

const formatCurrency = (value: number | undefined) =>
    `${new Intl.NumberFormat('en-US').format(Number(value ?? 0))} VND`

const formatDate = (value: string | undefined) => {
    if (!value) return ''
    const date = new Date(value)
    const day = String(date.getDate()).padStart(2, '0')
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const year = String(date.getFullYear()).slice(-2)
    return `${day}-${month}-${year}`
}

onMounted(async () => {
    try {
        const response = await axios.get(`/api/orders/${route.params.mahd}`, { withCredentials: true })
        order.value = response.data
    } catch (error) {
        errorMessage.value = axios.isAxiosError(error)
            ? error.response?.data?.message || 'Không thể tải đơn hàng.'
            : 'Không thể tải đơn hàng.'
        console.error(error)
    } finally {
        isLoading.value = false
    }
})
</script>

<template>
    <main class="main order-success-page">
        <div class="container">
            <div v-if="isLoading" class="status-message">Đang tải đơn hàng...</div>
            <div v-else-if="errorMessage" class="alert alert-danger order-success-content">{{ errorMessage }}</div>

            <div v-else-if="order" class="order-success-content">
                <div class="success-banner">
                    <strong><i class="fas fa-check"></i> Đặt hàng thành công.</strong>
                </div>

                <div class="order-meta">
                    <div><span>Mã đơn hàng</span><strong>{{ order.mahd }}</strong></div>
                    <div><span>Người nhận</span><strong>{{ receiverName }}</strong></div>
                    <div><span>Thời gian</span><strong>{{ formatDate(order.ngaylap) }}</strong></div>
                    <div><span>Thanh toán</span><strong>{{ formatCurrency(order.thanhtien) }}</strong></div>
                    <div><span>Phương thức</span><strong>{{ order.htthanhtoan }}</strong></div>
                </div>

                <div class="order-card">
                    <h3>ĐƠN HÀNG</h3>
                    <table class="order-table">
                        <thead>
                            <tr>
                                <th colspan="2">Sản phẩm</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="detail in order.details" :key="detail.masp">
                                <td>{{ detail.tensp || detail.masp }} <span>× {{ detail.soluong }}</span></td>
                                <td>{{ formatCurrency(detail.gia * detail.soluong) }}</td>
                            </tr>
                        </tbody>
                        <tfoot>
                            <tr>
                                <th>Tạm tính</th>
                                <td>{{ formatCurrency(order.subtotal) }}</td>
                            </tr>
                            <tr>
                                <th>Tiền ship</th>
                                <td>+ {{ formatCurrency(order.shippingFee) }}</td>
                            </tr>
                            <tr>
                                <th>Tiền khuyến mãi <small v-if="order.makm">({{ order.makm }})</small></th>
                                <td>- {{ formatCurrency(order.discount) }}</td>
                            </tr>
                            <tr class="grand-total">
                                <th>Thanh toán</th>
                                <td>{{ formatCurrency(order.thanhtien) }}</td>
                            </tr>
                        </tfoot>
                    </table>
                </div>

                <div class="address-grid">
                    <section>
                        <h4>Địa chỉ gửi hàng</h4>
                        <ul>
                            <li>Shop Anime Tôm Ká</li>
                            <li>Đường Hoàng Quốc Việt</li>
                            <li>Hà Nội</li>
                            <li>123 456 7890</li>
                        </ul>
                    </section>
                    <section>
                        <h4>Địa chỉ giao hàng</h4>
                        <ul>
                            <li>{{ receiverName }}</li>
                            <li>{{ receiverAddress }}</li>
                            <li>{{ receiverProvince }}</li>
                            <li>{{ receiverPhone }}</li>
                        </ul>
                    </section>
                </div>

                <div class="continue-shopping">
                    <router-link to="/" class="btn btn-primary">
                        <i class="fas fa-shopping-bag"></i> Tiếp tục mua sắm
                    </router-link>
                </div>
            </div>
        </div>
    </main>
</template>

<style scoped>
.order-success-page {
    min-height: 70vh;
    padding: 2.2rem 0 5rem;
    color: #222529;
}

.order-success-content {
    width: 100%;
    max-width: 650px;
    margin: 0 auto;
}

.status-message {
    padding: 5rem;
    text-align: center;
}

.success-banner {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 116px;
    border: 2px solid #0cc485;
    background: #fff;
    font-size: 1.6rem;
}

.success-banner i {
    margin-right: 0.5rem;
    color: #111;
}

.order-meta {
    display: grid;
    grid-template-columns: 1fr 1fr 0.9fr 1.15fr 1.45fr;
    gap: 1rem;
    margin: 2.8rem 0 2.4rem;
}

.order-meta div {
    min-width: 0;
    text-align: center;
}

.order-meta span,
.order-meta strong {
    display: block;
}

.order-meta span {
    margin-bottom: 0.5rem;
    color: #777;
    font-size: 1.15rem;
}

.order-meta strong {
    color: #111;
    font-size: 1.2rem;
    line-height: 1.35;
}

.order-card {
    padding: 2.8rem 2.8rem 3.4rem;
    border: 1px solid #ddd;
    background: #fff;
}

.order-card h3 {
    margin: 0 0 3rem;
    color: #666;
    font-size: 1.6rem;
}

.order-table {
    width: 100%;
    border-collapse: collapse;
}

.order-table th,
.order-table td {
    padding: 1.3rem 1rem;
    border: 0;
    font-size: 1.2rem;
}

.order-table th {
    color: #111;
    text-align: left;
}

.order-table td:last-child {
    color: #777;
    text-align: right;
    white-space: nowrap;
}

.order-table tbody td {
    padding-top: 0.7rem;
    padding-bottom: 1.8rem;
}

.order-table tbody td:first-child {
    color: #111;
}

.order-table tfoot tr:not(.grand-total) th,
.order-table tfoot tr:not(.grand-total) td {
    padding-top: 1.5rem;
    padding-bottom: 1.5rem;
}

.order-table .grand-total {
    border-top: 1px solid #ddd;
}

.order-table .grand-total th,
.order-table .grand-total td {
    padding-top: 1.5rem;
    color: #111;
    font-size: 1.8rem;
    font-weight: 700;
}

.address-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.8rem;
    padding-top: 3.8rem;
}

.address-grid h4 {
    margin-bottom: 1rem;
    color: #111;
    font-size: 1.7rem;
}

.address-grid ul {
    margin: 0;
    padding: 0;
    list-style: none;
}

.address-grid li {
    position: relative;
    min-height: 32px;
    padding: 0.7rem 0.5rem 0.7rem 1.2rem;
    border-bottom: 1px solid #ddd;
    color: #111;
    font-size: 1.15rem;
}

.address-grid li::before {
    position: absolute;
    left: 0;
    content: "›";
}

.continue-shopping {
    margin-top: 2.4rem;
    text-align: right;
}

.continue-shopping .btn {
    padding: 1.2rem 2rem;
    border-radius: 0;
}

@media (max-width: 767px) {
    .order-meta {
        grid-template-columns: repeat(2, 1fr);
    }

    .order-card {
        padding: 2rem 1.5rem;
    }
}

@media (max-width: 575px) {
    .address-grid {
        grid-template-columns: 1fr;
    }
}
</style>
