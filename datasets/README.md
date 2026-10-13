# Extraction Benchmark Datasets

Saved Tue Oct 13 2026. ~3.5 GB total. Use for testing/benchmarking small document-extraction models (NuExtract3, ReaderLM-v2, GLiNER2, etc.).

| # | Dir | Source | Size | Docs | What it tests |
|---|---|---|---|---|---|
| 01 | `01_invoice_extract_benchmark` | HF: drew-ipp/invoice-extraction-benchmark | 36 MB | ~30 synth invoices | Synthetic invoice PDFs + ground-truth JSON: vendor, invoice #, dates, currency, line items, totals, taxes. Includes a **scoring harness** (corpus/generator/scorer). Perfect micro-benchmark for our exact use case. |
| 02 | `02_nuextract_github_repo` | github.com/numindai/nuextract | 1 MB | n/a | The model repo. The official benchmark (600 docs) is **not public yet** — NuMind says it will be open-sourced "in the coming weeks" per their README. |
| 03 | `03_invoice_extraction_v2` | HF: manuelaschrittwieser/invoice-extraction-dataset-v2 | <1 MB | 601 | JSONL with messy text → JSON. Built for **fine-tuning**, but useful as a held-out eval set. Schema: vendor, date, line items, totals. |
| 04 | `04_sroie` | HF: jsdnrs/ICDAR2019-SROIE | 486 MB | 1,000 (626 train / 347 test) | Scanned restaurant receipts. 4 fields: company, address, date, total. Images + OCR. **Classic KIE baseline**. ⚠️ Has ~75% template overlap train↔test. |
| 05 | `05_cord` | HF: naver-clova-ix/cord-v2 | 2.2 GB | 1,000 (800/100/100) | Receipts with **30+ line-item fields** (menu items, prices, totals, payment). Best for line-item extraction. |
| 06 | `06_funsd` | HF: nielsr/FUNSD | 16 MB | 199 (149 train / 50 test) | Noisy scanned forms. Word-level + entity-level annotations. 4 labels: question / answer / header / other. Tests robustness on low-quality scans. |
| 07 | `07_extractbench` | HF: llamaindex/ExtractBench | 786 MB | **370 (252 short / 98 medium / 20 long)** | **The best fit for our case.** Real enterprise docs (tax forms, contracts, invoices, financial statements). Each doc has a custom JSON schema and ground truth with bbox evidence. Split by document length so you can score on short/medium/long separately. Apache-2.0. Has eval harness at github.com/run-llama/extract-bench. |

## Total available: ~3.5 GB / 5,055 docs

## Datasets NOT downloaded (and why)

- **NuExtract-tests (numindai/NuExtract-tests)** — Not on HuggingFace. NuMind's official 600-doc benchmark is private; they say they'll release it. Watch `numindai/nuextract` GitHub.
- **LLMStructBench (Tenckhoff et al.)** — Not yet on HuggingFace. Their website says "dataset and benchmark code will follow". Paper: arXiv 2602.14743.

## Recommended quick-start benchmark

For "pricing/dates/line items from long docs" use:

1. **ExtractBench short split** (252 docs, mostly tax forms / invoices) — fastest signal, has full ground truth + bbox evidence
2. **drew-ipp/invoice-extraction-benchmark** — purpose-built for invoice extraction, includes its own scorer
3. **CORD** — only if you specifically care about line-item extraction at scale

Once you've picked a model and schema, validate on ~20 of your own real production docs.

## Reproduction commands

```bash
# Pulled via huggingface_hub 1.24.0 + hf CLI
hf download drew-ipp/invoice-extraction-benchmark            --repo-type dataset --local-dir ./01_invoice_extract_benchmark
hf download jsdnrs/ICDAR2019-SROIE                          --repo-type dataset --local-dir ./04_sroie
hf download naver-clova-ix/cord-v2                          --repo-type dataset --local-dir ./05_cord
hf download nielsr/FUNSD                                    --repo-type dataset --local-dir ./06_funsd
hf download llamaindex/ExtractBench                         --repo-type dataset --local-dir ./07_extractbench
hf download manuelaschrittwieser/invoice-extraction-dataset-v2 --repo-type dataset --local-dir ./03_invoice_extraction_v2

# The NuExtract github clone (no public test set on HF):
git clone --depth 1 https://github.com/numindai/nuextract.git 02_nuextract_github_repo
```

## File layout
```
datasets/
├── _logs/                              # download logs
├── 01_invoice_extract_benchmark/       # synthetic invoices + scorer
├── 02_nuextract_github_repo/           # model repo (no eval data)
├── 03_invoice_extraction_v2/           # 601 messy→JSON pairs (JSONL/parquet)
├── 04_sroie/                           # 1,000 receipts (parquets)
├── 05_cord/                            # 1,000 receipts with line items (parquets)
├── 06_funsd/                           # 199 noisy forms (parquets)
└── 07_extractbench/                    # 370 enterprise docs (pdfs + jsonl)
```