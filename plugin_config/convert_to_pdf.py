#!/usr/bin/env python3
import markdown_pdf
from pathlib import Path

# Read the markdown file
md_content = Path("PATENT_OVERVIEW.md").read_text()

# Create PDF with proper configuration
pdf = markdown_pdf.MarkdownPdf(toc_level=3)
pdf.add_section(markdown_pdf.Section(md_content, toc=False))
pdf.meta["title"] = "BLE Profile Library - Technical Documentation"
pdf.meta["author"] = "Wanyeki Technologies LLC"
pdf.save("PATENT_OVERVIEW.pdf")

print("PDF created successfully!")
