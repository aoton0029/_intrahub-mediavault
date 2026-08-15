"""Generate the tiny deterministic binary fixtures committed beside this script."""

from __future__ import annotations

import io
import zlib
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).parent


def _font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = (
        Path("C:/Windows/Fonts/msgothic.ttc"),
        Path("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        Path("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
    )
    for candidate in candidates:
        if candidate.exists():
            return ImageFont.truetype(str(candidate), size)
    return ImageFont.load_default()


def _page_image(text: str) -> Image.Image:
    image = Image.new("RGB", (640, 240), "white")
    ImageDraw.Draw(image).text((35, 80), text, fill="black", font=_font(38))
    return image


def _pdf(objects: list[bytes]) -> bytes:
    output = bytearray(b"%PDF-1.4\n%fixture\n")
    offsets = [0]
    for number, body in enumerate(objects, 1):
        offsets.append(len(output))
        output.extend(f"{number} 0 obj\n".encode())
        output.extend(body)
        output.extend(b"\nendobj\n")
    xref = len(output)
    output.extend(f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode())
    output.extend(b"".join(f"{offset:010d} 00000 n \n".encode() for offset in offsets[1:]))
    output.extend(
        f"trailer << /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    )
    return bytes(output)


def _make_pdf(page_kinds: list[str], path: Path) -> None:
    # Objects 1/2 are catalog/pages. Each page then owns page/content and optional image objects.
    objects: list[bytes] = [b"<< /Type /Catalog /Pages 2 0 R >>", b""]
    page_refs: list[int] = []
    for index, kind in enumerate(page_kinds, 1):
        page_number = len(objects) + 1
        content_number = page_number + 1
        page_refs.append(page_number)
        if kind == "text":
            text = f"MediaVault text layer fixture page {index}. " * 4
            stream = f"BT /F1 14 Tf 40 160 Td ({text}) Tj ET".encode()
            objects.append(
                (
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 640 240] "
                    "/Resources << /Font << /F1 << /Type /Font /Subtype /Type1 "
                    f"/BaseFont /Helvetica >> >> >> /Contents {content_number} 0 R >>"
                ).encode()
            )
            objects.append(
                f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"\nendstream"
            )
        else:
            image = _page_image(f"日本語 OCR テスト {index}")
            raw = image.tobytes()
            compressed = zlib.compress(raw)
            image_number = content_number + 1
            stream = b"q 640 0 0 240 0 0 cm /Im0 Do Q"
            objects.append(
                (
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 640 240] "
                    f"/Resources << /XObject << /Im0 {image_number} 0 R >> >> "
                    f"/Contents {content_number} 0 R >>"
                ).encode()
            )
            objects.append(
                f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"\nendstream"
            )
            objects.append(
                (
                    "<< /Type /XObject /Subtype /Image /Width 640 /Height 240 "
                    "/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode "
                    f"/Length {len(compressed)} >>\nstream\n"
                ).encode()
                + compressed
                + b"\nendstream"
            )
    kids = " ".join(f"{ref} 0 R" for ref in page_refs)
    objects[1] = f"<< /Type /Pages /Count {len(page_refs)} /Kids [{kids}] >>".encode()
    path.write_bytes(_pdf(objects))


def main() -> None:
    japanese = _page_image("日本語 OCR テスト")
    japanese.save(HERE / "japanese.png", optimize=True)
    _make_pdf(["text", "text", "text"], HERE / "text_layer.pdf")
    _make_pdf(["image", "image", "image"], HERE / "scanned.pdf")
    _make_pdf(["text", "image", "text"], HERE / "mixed.pdf")
    (HERE / "corrupt.pdf").write_bytes(b"%PDF-1.4\nthis fixture is intentionally corrupt\n")
    buffer = io.BytesIO()
    japanese.save(buffer, format="PNG", optimize=True)
    (HERE / "fake.pdf").write_bytes(buffer.getvalue())


if __name__ == "__main__":
    main()
