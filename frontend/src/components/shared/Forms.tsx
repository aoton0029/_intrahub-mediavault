import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

export function FormSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h3 className="form-section-title">{title}</h3>
      {children}
    </section>
  );
}

export function FormGrid({ children }: { children: ReactNode }) {
  return <div className="form-grid">{children}</div>;
}

export function FormField({
  label,
  error,
  hint,
  full = false,
  required = false,
  children,
}: {
  label: string;
  error?: string;
  hint?: string;
  full?: boolean;
  required?: boolean;
  children: ReactNode;
}) {
  return (
    <div className={cn("form-field", full && "full", error && "error")}>
      <label>
        {label}
        {required ? <span className="required">*</span> : null}
      </label>
      {children}
      {error ? <span className="field-error">{error}</span> : null}
      {hint ? <span className="field-hint">{hint}</span> : null}
    </div>
  );
}

export function FormActions({ children }: { children: ReactNode }) {
  return <div className="form-actions">{children}</div>;
}
