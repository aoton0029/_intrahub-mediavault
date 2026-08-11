import type { ReactNode } from "react";

export function Modal({
  open,
  onClose,
  title,
  children,
  maxWidth,
  height,
}: {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  maxWidth?: number | string;
  height?: number | string;
}) {
  if (!open) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className={height ? "modal is-fixed-height" : "modal"}
        style={maxWidth || height ? { maxWidth, height } : undefined}
        onClick={(event) => event.stopPropagation()}
      >
        <h2>{title}</h2>
        {children}
      </div>
    </div>
  );
}
