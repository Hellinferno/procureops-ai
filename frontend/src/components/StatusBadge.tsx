import { Check, Clock3, Ban, CircleAlert, ArrowUpRight } from "lucide-react";
import type { TransactionStatus } from "../types";

const statuses: Record<TransactionStatus, { label: string; tone: string; icon: typeof Check }> = {
  COMPLETED: { label: "Captured", tone: "success", icon: Check },
  SETTLED: { label: "Settled", tone: "success", icon: Check },
  BLOCKED_BY_GUARDRAIL: { label: "Policy blocked", tone: "danger", icon: Ban },
  APPROVED_PENDING_SETTLEMENT: { label: "Pending settlement", tone: "neutral", icon: Clock3 },
  AWAITING_APPROVAL: { label: "Needs approval", tone: "warning", icon: Clock3 },
  PAYMENT_PENDING: { label: "Payment pending", tone: "warning", icon: Clock3 },
  PAYMENT_FAILED: { label: "Payment failed", tone: "danger", icon: CircleAlert },
  CANCELLED: { label: "Cancelled", tone: "neutral", icon: Ban },
  PAYOUT_PENDING: { label: "Payout pending", tone: "info", icon: ArrowUpRight },
  PAYOUT_FAILED: { label: "Payout failed", tone: "danger", icon: CircleAlert },
};
export const statusLabel = (status: TransactionStatus) => statuses[status].label;
export default function StatusBadge({ status }: { status: TransactionStatus }) {
  const { label, tone, icon: Icon } = statuses[status];
  return <span className={`status-badge ${tone}`} title={status}><Icon size={12} aria-hidden="true" />{label}</span>;
}
