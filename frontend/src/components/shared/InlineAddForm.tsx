import { useState, type ReactNode } from "react";
import { FiPlus } from "react-icons/fi";

export type InlineField = {
  name: string;
  placeholder: string;
  type?: "text" | "number" | "select";
  options?: { value: string; label: string }[];
  defaultValue?: string;
};

function initialValues(fields: InlineField[]) {
  return Object.fromEntries(fields.map((field) => [field.name, field.defaultValue ?? ""]));
}

export function InlineAddForm({
  triggerLabel,
  icon,
  fields,
  onSubmit,
}: {
  triggerLabel: string;
  icon?: ReactNode;
  fields: InlineField[];
  onSubmit: (values: Record<string, string>) => void;
}) {
  const [isAdding, setIsAdding] = useState(false);
  const [values, setValues] = useState<Record<string, string>>(() => initialValues(fields));

  function closeForm() {
    setIsAdding(false);
    setValues(initialValues(fields));
  }

  function submit() {
    onSubmit(values);
    closeForm();
  }

  if (!isAdding) {
    return (
      <button type="button" className="btn btn-ghost btn-sm" onClick={() => setIsAdding(true)}>
        {icon ?? <FiPlus className="icon" />}
        {triggerLabel}
      </button>
    );
  }

  return (
    <div className="inline-add-form" style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
      {fields.map((field) =>
        field.type === "select" ? (
          <select
            key={field.name}
            value={values[field.name]}
            onChange={(event) => setValues((prev) => ({ ...prev, [field.name]: event.target.value }))}
          >
            {field.options?.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        ) : (
          <input
            key={field.name}
            type={field.type === "number" ? "number" : "text"}
            value={values[field.name]}
            placeholder={field.placeholder}
            autoFocus={field === fields[0]}
            onChange={(event) => setValues((prev) => ({ ...prev, [field.name]: event.target.value }))}
            onKeyDown={(event) => {
              if (event.key === "Enter" && fields.length === 1) submit();
              if (event.key === "Escape") closeForm();
            }}
          />
        ),
      )}
      <button type="button" className="btn btn-primary btn-sm" onClick={submit}>
        追加
      </button>
      <button type="button" className="btn btn-ghost btn-sm" onClick={closeForm}>
        キャンセル
      </button>
    </div>
  );
}
