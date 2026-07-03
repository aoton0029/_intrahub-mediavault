import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"

import { Button } from "./button"

describe("Button", () => {
  // 【正常系】variantごとのクラス付与
  it("TC-BTN-N-01: variant未指定時にデフォルト(.btn相当)のスタイルが付与される", () => {
    render(<Button>OK</Button>)
    const button = screen.getByRole("button", { name: "OK" })
    expect(button.dataset.variant).toBe("default")
    expect(button.className).toContain("bg-card")
    expect(button.className).toContain("text-card-foreground")
    expect(button.className).toContain("border-border")
  })

  it("TC-BTN-N-02: variant=accent指定時にアクセント配色のスタイルが付与される", () => {
    render(<Button variant="accent">追加</Button>)
    const button = screen.getByRole("button", { name: "追加" })
    expect(button.dataset.variant).toBe("accent")
    expect(button.className).toContain("bg-primary")
    expect(button.className).toContain("text-white")
  })

  it("TC-BTN-N-03: variant=ghost指定時にghost配色のスタイルが付与される", () => {
    render(<Button variant="ghost">キャンセル</Button>)
    const button = screen.getByRole("button", { name: "キャンセル" })
    expect(button.dataset.variant).toBe("ghost")
    expect(button.className).toContain("text-muted-foreground")
  })

  it("TC-BTN-N-04: variant=danger指定時にdanger配色のスタイルが付与される", () => {
    render(<Button variant="danger">削除</Button>)
    const button = screen.getByRole("button", { name: "削除" })
    expect(button.dataset.variant).toBe("danger")
    expect(button.className).toContain("text-destructive")
    expect(button.className).toContain("bg-card")
  })

  // 【正常系】sizeごとのクラス付与
  it("TC-BTN-N-05: size=sm指定時に.btn-sm相当のpadding/font-sizeが適用される", () => {
    render(<Button size="sm">保存</Button>)
    const button = screen.getByRole("button", { name: "保存" })
    expect(button.dataset.size).toBe("sm")
    expect(button.className).toContain("px-2.5")
    expect(button.className).toContain("py-1")
    expect(button.className).toContain("text-xs")
  })

  // 【境界値】フォーカスリング
  it("TC-BTN-B-01: フォーカス時にアクセント色相当のリングクラスが付与される", () => {
    render(<Button>OK</Button>)
    const button = screen.getByRole("button", { name: "OK" })
    expect(button.className).toContain("focus-visible:ring-ring/50")
  })

  // 【正常系】既存shadcn標準variantが維持されていること
  it("TC-BTN-N-06: 既存のoutline/secondary/destructive/link variantが引き続き利用できる", () => {
    render(
      <>
        <Button variant="outline">outline</Button>
        <Button variant="secondary">secondary</Button>
        <Button variant="destructive">destructive</Button>
        <Button variant="link">link</Button>
      </>
    )
    expect(screen.getByRole("button", { name: "outline" }).dataset.variant).toBe("outline")
    expect(screen.getByRole("button", { name: "secondary" }).dataset.variant).toBe("secondary")
    expect(screen.getByRole("button", { name: "destructive" }).dataset.variant).toBe("destructive")
    expect(screen.getByRole("button", { name: "link" }).dataset.variant).toBe("link")
  })
})
