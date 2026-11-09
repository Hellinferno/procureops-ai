/** ProcureOps' routed P: a purchase loop crossed by an approval check. */
export default function ProcureMark({ className = "" }: { className?: string }) {
  return (
    <svg
      className={`procure-mark ${className}`}
      viewBox="0 0 64 64"
      fill="none"
      aria-hidden="true"
      focusable="false"
    >
      <path
        className="procure-mark-core"
        fillRule="evenodd"
        clipRule="evenodd"
        d="M8 8h28l16 16v12L36 52H24v10H8V8Zm16 15v14h7l8-8-6-6h-9Z"
      />
      <path
        className="procure-mark-check"
        d="m24 44 12 12 23-23-6-6-17 17-6-6-6 6Z"
      />
    </svg>
  );
}
