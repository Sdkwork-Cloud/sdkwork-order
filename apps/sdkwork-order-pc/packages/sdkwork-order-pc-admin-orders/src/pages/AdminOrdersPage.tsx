import { useEffect, useMemo, useState } from "react";
import { Button, EmptyState, LoadingBlock } from "@sdkwork/ui-pc-react";
import { getSdkworkOrderBackendSdkClient } from "@sdkwork/order-service";
import type { OrderDetail, OrderSummary } from "@sdkwork/order-backend-sdk";
import { createOrderAdminService } from "../order-admin-service";

const DEFAULT_PAGE_SIZE = 20;

export function SdkworkOrderAdminOrdersPage() {
  const service = useMemo(
    () => createOrderAdminService(getSdkworkOrderBackendSdkClient()),
    [],
  );
  const [orders, setOrders] = useState<OrderSummary[]>([]);
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [totalPages, setTotalPages] = useState(1);
  const [loading, setLoading] = useState(true);
  const [listError, setListError] = useState<string | null>(null);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<OrderDetail | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    async function loadOrders() {
      setLoading(true);
      setListError(null);
      try {
        const result = await service.listOrders({
          page,
          pageSize: DEFAULT_PAGE_SIZE,
          status: statusFilter.trim() || undefined,
          q: searchQuery.trim() || undefined,
        });
        if (active) {
          setOrders(result.items);
          setTotalPages(Math.max(1, result.totalPages));
        }
      } catch {
        if (active) {
          setListError("订单列表加载失败，请检查 commerce.orders.read 权限与网络连接");
          setOrders([]);
        }
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    }
    void loadOrders();
    return () => {
      active = false;
    };
  }, [page, searchQuery, service, statusFilter]);

  useEffect(() => {
    if (!selectedId) {
      setDetail(null);
      setDetailError(null);
      return;
    }
    let active = true;
    setDetailLoading(true);
    setDetailError(null);
    void service
      .getOrder(selectedId)
      .then((value: OrderDetail) => {
        if (active) {
          setDetail(value);
        }
      })
      .catch(() => {
        if (active) {
          setDetail(null);
          setDetailError("订单详情加载失败");
        }
      })
      .finally(() => {
        if (active) {
          setDetailLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, [selectedId, service]);

  async function mutateOrder(orderId: string, action: "cancel" | "close") {
    setBusyId(orderId);
    setMessage(null);
    try {
      if (action === "cancel") {
        await service.cancelOrder(orderId);
      } else {
        await service.closeOrder(orderId);
      }
      setMessage(`订单 ${orderId} 已${action === "cancel" ? "取消" : "关闭"}`);
      const result = await service.listOrders({
        page,
        pageSize: DEFAULT_PAGE_SIZE,
        status: statusFilter.trim() || undefined,
        q: searchQuery.trim() || undefined,
      });
      setOrders(result.items);
      setTotalPages(Math.max(1, result.totalPages));
    } catch {
      setMessage("操作失败，请检查 commerce.orders.manage 权限与订单状态");
    } finally {
      setBusyId(null);
    }
  }

  if (loading && orders.length === 0) {
    return <LoadingBlock label="加载订单..." />;
  }

  return (
    <div className="order-admin-page">
      <h1>订单监管</h1>
      <p className="order-admin-hint">需要 IAM 权限：commerce.orders.read（查看）、commerce.orders.manage（取消/关闭）</p>
      <div className="order-admin-filters">
        <label>
          状态
          <input
            type="search"
            value={statusFilter}
            onChange={(event) => {
              setPage(1);
              setStatusFilter(event.target.value);
            }}
            placeholder="pending_payment"
          />
        </label>
        <label>
          搜索
          <input
            type="search"
            value={searchQuery}
            onChange={(event) => {
              setPage(1);
              setSearchQuery(event.target.value);
            }}
            placeholder="订单号 / 主题"
          />
        </label>
      </div>
      {listError ? <p role="alert">{listError}</p> : null}
      {message ? <p role="status">{message}</p> : null}
      {orders.length === 0 && !listError ? (
        <EmptyState description="平台订单将在此展示" title="暂无订单" />
      ) : (
        <>
          <table className="order-admin-table">
            <thead>
              <tr>
                <th>订单</th>
                <th>状态</th>
                <th>金额</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {orders.map((order) => (
                <tr key={order.orderId}>
                  <td>
                    <button onClick={() => setSelectedId(order.orderId)} type="button">
                      {order.subject || order.orderSn}
                    </button>
                  </td>
                  <td>{order.statusName || order.status}</td>
                  <td>{order.totalAmount}</td>
                  <td className="order-admin-actions">
                    <Button disabled={busyId === order.orderId} onClick={() => setSelectedId(order.orderId)} type="button">
                      详情
                    </Button>
                    <Button disabled={busyId === order.orderId} onClick={() => void mutateOrder(order.orderId, "cancel")} type="button">
                      取消
                    </Button>
                    <Button disabled={busyId === order.orderId} onClick={() => void mutateOrder(order.orderId, "close")} type="button">
                      关闭
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="order-admin-pagination">
            <Button disabled={page <= 1 || loading} onClick={() => setPage((value) => value - 1)} type="button">
              上一页
            </Button>
            <span>
              第 {page} / {totalPages} 页
            </span>
            <Button disabled={page >= totalPages || loading} onClick={() => setPage((value) => value + 1)} type="button">
              下一页
            </Button>
          </div>
        </>
      )}

      {selectedId ? (
        <section className="order-admin-detail">
          <h2>订单详情</h2>
          {detailLoading ? <p>加载详情...</p> : null}
          {detailError ? <p role="alert">{detailError}</p> : null}
          {detail ? (
            <>
              <p>订单号：{detail.orderSn}</p>
              <p>状态：{detail.statusName || detail.status}</p>
              <p>金额：{detail.totalAmount}</p>
              <p>数量：{detail.quantity}</p>
            </>
          ) : null}
        </section>
      ) : null}
    </div>
  );
}
