<template>
  <section class="card order-list-card">
    <div class="list-header">
      <div>
        <strong>Danh sách hóa đơn</strong>
        <span>{{ total }} hóa đơn</span>
      </div>
    </div>

    <div class="control-panel order-control-panel">
      <input v-model.trim="keyword" class="search" placeholder="Tìm theo mã, khách hàng hoặc địa chỉ..."
        @keyup.enter="applyFilters" />
      <select v-model="statusFilter" @change="applyFilters">
        <option value="">Tất cả trạng thái</option>
        <option v-for="s in statuses" :key="s.matt" :value="s.matt">{{ s.tentrangthai }}</option>
      </select>
      <select v-model="paymentFilter" @change="applyFilters">
        <option value="">Tất cả thanh toán</option>
        <option>Thanh toán khi nhận hàng</option>
        <option>Chuyển khoản ngân hàng</option>
      </select>
      <select v-model="sort" @change="applyFilters">
        <option value="newest">Mới nhất</option>
        <option value="oldest">Cũ nhất</option>
        <option value="amount_desc">Giá trị cao</option>
        <option value="amount_asc">Giá trị thấp</option>
      </select>
      <button class="btn ghost" :disabled="loading" @click="resetAndLoad">{{ loading ? 'Đang tải...' : 'Tải lại'
        }}</button>
    </div>
    <div v-if="error" class="alert error">{{ error }}</div>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Mã</th>
            <th>Ngày lập</th>
            <th>Khách hàng</th>
            <th>Địa chỉ</th>
            <th>Thanh toán</th>
            <th>Tổng tiền</th>
            <th>Trạng thái</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="order in filtered" :key="order.mahd" class="clickable-row" tabindex="0"
            @click="openOrder(order.mahd)" @keydown.enter.prevent="openOrder(order.mahd)"
            @keydown.space.prevent="openOrder(order.mahd)">
            <td><strong>{{ order.mahd }}</strong></td>
            <td>{{ formatDate(order.ngaylap) }}</td>
            <td>{{ order.tenkh || order.mand || '-' }}</td>
            <td>{{ order.diachi || '-' }}</td>
            <td>{{ order.htthanhtoan || '-' }}</td>
            <td>{{ formatMoney(order.thanhtien) }}</td>
            <td><span class="badge order-status" :class="statusClass(order)">{{ order.trangthai || 'Chưa có' }}</span></td>
            <td><button class="btn ghost small" @click.stop="openOrder(order.mahd)">Chi tiết</button></td>
          </tr>
          <tr v-if="!loading && filtered.length === 0">
            <td colspan="8" class="empty-cell">{{ emptyMessage }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <Pagination :page="page" :page-size="pageSize" :total="total" :total-pages="totalPages" @update:page="changePage"
      @update:page-size="changePageSize" />
  </section>

  <div v-if="drawerOpen" class="drawer" role="dialog" aria-modal="true" @click.self="closeDrawer">
    <div class="drawer-panel">
      <div class="drawer-header">
        <div>
          <h2>{{ selected ? `Hóa đơn ${selected.order.mahd}` : 'Đang tải...' }}</h2>
        </div>
        <button class="btn ghost" @click="closeDrawer">Đóng</button>
      </div>
      <div v-if="detailLoading" class="detail-loading">Đang tải chi tiết hóa đơn...</div>
      <template v-else-if="selected">
        <div v-if="detailError" class="alert error">{{ detailError }}</div>
        <div v-if="success" class="alert success">{{ success }}</div>
        <div class="order-summary-grid">
          <div><span>Khách hàng</span><strong>{{ selected.order.tenkh || selected.order.mand || '-' }}</strong></div>
          <div><span>Ngày lập</span><strong>{{ formatDate(selected.order.ngaylap) }}</strong></div>
          <div><span>Thanh toán</span><strong>{{ selected.order.htthanhtoan || '-' }}</strong></div>
          <div><span>Tổng tiền</span><strong>{{ formatMoney(selected.order.thanhtien) }}</strong></div>
          <div class="full"><span>Địa chỉ nhận hàng</span><strong>{{ selected.order.diachi || '-' }}</strong></div>
        </div>
        <section class="detail-section">
          <h3>Cập nhật trạng thái</h3>
          <div class="status-options" role="radiogroup" aria-label="Chọn trạng thái mới">
            <button v-for="(status, index) in statuses" :key="status.matt" type="button" class="status-option"
              :class="[statusClass(status), { active: newStatus === status.matt, current: selected.order.matt === status.matt, passed: isStatusLocked(index) }]"
              :disabled="saving || isStatusLocked(index)" :aria-pressed="newStatus === status.matt"
              @click="newStatus = status.matt">
              <span class="status-step">{{ index + 1 }}</span>
              <span>{{ status.tentrangthai || status.matt }}</span>
              <small v-if="selected.order.matt === status.matt">Hiện tại</small>
              <small v-else-if="isStatusLocked(index)">Đã qua</small>
            </button>
          </div>
          <div class="status-actions">
            <span class="muted">Chỉ có thể chuyển sang trạng thái tiếp theo.</span>
            <button class="btn primary" :disabled="!canUpdate" @click="updateStatus">{{ saving ? 'Đang lưu...' : 'Cập nhật trạng thái' }}</button>
          </div>
        </section>
        <section class="detail-section">
          <h3>Sản phẩm ({{ selected.items.length }})</h3>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Mã SP</th>
                  <th>Tên sản phẩm</th>
                  <th>Đơn giá</th>
                  <th>SL</th>
                  <th>Thành tiền</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(item, index) in selected.items" :key="`${item.masp}-${index}`">
                  <td>{{ item.masp || '-' }}</td>
                  <td>{{ item.tensp || '-' }}</td>
                  <td>{{ formatMoney(item.gia) }}</td>
                  <td>{{ item.soluong || 0 }}</td>
                  <td><strong>{{ formatMoney((item.gia || 0) * (item.soluong || 0)) }}</strong></td>
                </tr>
                <tr v-if="selected.items.length === 0">
                  <td colspan="5" class="empty-cell">Hóa đơn chưa có sản phẩm.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
        <section class="detail-section">
          <h3>Lịch sử trạng thái</h3>
          <ol v-if="selected.status_history.length" class="status-history">
            <li v-for="(status, index) in selected.status_history"
              :key="`${status.matt}-${status.ngaycapnhat}-${index}`"><span class="history-dot" :class="statusClass(status)"></span>
              <div><strong>{{ status.tentrangthai || status.matt || 'Không xác định' }}</strong><small>{{
                formatDateTime(status.ngaycapnhat) }}</small></div>
            </li>
          </ol>
          <p v-else class="muted">Chưa có lịch sử trạng thái.</p>
        </section>
      </template>
      <div v-else-if="detailError" class="alert error">{{ detailError }}</div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, formatDate, formatMoney } from '../api/http'
import Pagination from '../components/Pagination.vue'

const route = useRoute()
const router = useRouter()
const items = ref([]), statuses = ref([]), selected = ref(null), keyword = ref(''), newStatus = ref('')
const loading = ref(false), detailLoading = ref(false), saving = ref(false), drawerOpen = ref(false)
const error = ref(''), detailError = ref(''), success = ref('')
const page = ref(1), pageSize = ref(10), total = ref(0), totalPages = ref(0), statusFilter = ref(''), paymentFilter = ref(''), sort = ref('newest')
const filtered = computed(() => {
  const term = keyword.value.toLowerCase()
  return items.value.filter(order => [order.mahd, order.tenkh, order.mand, order.diachi, order.trangthai].some(value => String(value || '').toLowerCase().includes(term)))
})
const emptyMessage = computed(() => {
  if (keyword.value || statusFilter.value || paymentFilter.value) return 'Không tìm thấy hóa đơn phù hợp.'
  return 'Chưa có hóa đơn.'
})
const currentStatusIndex = computed(() => statuses.value.findIndex(status => status.matt === selected.value?.order.matt))
const selectedStatusIndex = computed(() => statuses.value.findIndex(status => status.matt === newStatus.value))
const canUpdate = computed(() =>
  Boolean(newStatus.value) &&
  (currentStatusIndex.value < 0 || selectedStatusIndex.value > currentStatusIndex.value) &&
  !saving.value
)
const getErrorMessage = (err, fallback) => err.response?.data?.message || fallback
const formatDateTime = value => value ? new Date(value).toLocaleString('vi-VN') : '-'
const isStatusLocked = index => currentStatusIndex.value >= 0 && index <= currentStatusIndex.value
function normalizeStatus(value) {
  return String(value || '').normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase()
}
function statusClass(status) {
  const raw = normalizeStatus(`${status?.matt || ''} ${status?.trangthai || ''} ${status?.tentrangthai || ''}`)
  if (!raw.trim()) return 'status-empty'
  if (raw.includes('huy') || raw.includes('cancel')) return 'status-cancelled'
  if (raw.includes('hoan') || raw.includes('thanh cong') || raw.includes('da giao') || raw.includes('done')) return 'status-done'
  if (raw.includes('giao') || raw.includes('van chuyen') || raw.includes('ship')) return 'status-shipping'
  if (raw.includes('xac nhan') || raw.includes('xu ly') || raw.includes('cho') || raw.includes('pending')) return 'status-pending'
  return 'status-processing'
}

async function load() {
  loading.value = true; error.value = ''
  try {
    const [ordersResponse, statusesResponse] = await Promise.all([api.get('/orders', { params: { page: page.value, page_size: pageSize.value, q: keyword.value, status: statusFilter.value, payment: paymentFilter.value, sort: sort.value } }), api.get('/statuses')])
    items.value = ordersResponse.data.items; total.value = ordersResponse.data.total; totalPages.value = ordersResponse.data.total_pages; statuses.value = statusesResponse.data
    openOrderFromQuery()
  } catch (err) { error.value = getErrorMessage(err, 'Không thể tải danh sách hóa đơn.') }
  finally { loading.value = false }
}
function applyFilters() { page.value = 1; load() } function changePage(value) { page.value = value; load() } function changePageSize(value) { pageSize.value = value; page.value = 1; load() }
function resetAndLoad() { keyword.value = ''; statusFilter.value = ''; paymentFilter.value = ''; sort.value = 'newest'; page.value = 1; load() }
async function openOrder(orderId) {
  drawerOpen.value = true; detailLoading.value = true; detailError.value = ''; success.value = ''; selected.value = null
  try {
    selected.value = (await api.get(`/orders/${encodeURIComponent(orderId)}`)).data
    selected.value.status_history ||= []; newStatus.value = selected.value.order.matt || ''
  } catch (err) { detailError.value = getErrorMessage(err, 'Không thể tải chi tiết hóa đơn.') }
  finally { detailLoading.value = false }
}
function closeDrawer() {
  if (saving.value) return
  drawerOpen.value = false; selected.value = null
  if (route.query.order) router.replace({ path: route.path, query: { ...route.query, order: undefined } })
}
function openOrderFromQuery() {
  const orderId = route.query.order
  if (typeof orderId === 'string' && orderId && selected.value?.order.mahd !== orderId) openOrder(orderId)
}
async function updateStatus() {
  if (!canUpdate.value) return
  saving.value = true; detailError.value = ''; success.value = ''
  const orderId = selected.value.order.mahd
  try {
    await api.post(`/orders/${encodeURIComponent(orderId)}/status`, { matt: newStatus.value })
    const detailResponse = await api.get(`/orders/${encodeURIComponent(orderId)}`)
    selected.value = detailResponse.data; selected.value.status_history ||= []; newStatus.value = selected.value.order.matt || ''
    await load()
    selected.value = detailResponse.data; selected.value.status_history ||= []; newStatus.value = selected.value.order.matt || ''
    success.value = 'Cập nhật trạng thái hóa đơn thành công.'
  } catch (err) { detailError.value = getErrorMessage(err, 'Không thể cập nhật trạng thái hóa đơn.') }
  finally { saving.value = false }
}
onMounted(load)
watch(() => route.query.order, openOrderFromQuery)
</script>
