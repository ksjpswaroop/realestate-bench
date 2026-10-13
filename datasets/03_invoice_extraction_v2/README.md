---
license: apache-2.0
task_categories:
- text-generation
- token-classification
language:
- en
tags:
- finance
- invoice-extraction
- synthetic
- structured-data
- json
- llama-3
pretty_name: Invoice Extraction Dataset v2
size_categories:
- n < 1K
---

# 📑 Invoice Extraction Dataset v2

This dataset is specifically designed for fine-tuning Large Language Models (LLMs) to perform **Structured Data Extraction**.
It contains 601 unique examples of messy, unstructured invoice and receipt descriptions paired with their clean, machine-readable JSON counterparts.

## 🚀 Dataset Purpose
General LLMs often include conversational filler when asked for JSON. This dataset was built to train models to:
1.  **Ignore "Noise":** Ignore conversational filler and irrelevant words.
2.  **Strict Formatting:** Output *only* valid JSON.
3.  **Entity Recognition:** Correctly identify vendors, totals, dates, and items regardless of sentence structure.

## 📊 Dataset Structure
The dataset is provided in **JSONL (JSON Lines)** format, which is the industry standard for LLM fine-tuning. Each row contains:

- `instruction`: The task description ("Extract invoice details into JSON.")
- `input`: A messy, natural language description of a transaction.
- `output`: A stringified JSON object containing the extracted fields.

### Target Schema
The model is trained to extract the following fields:
```json
{
  "item": "string",
  "quantity": "integer",
  "date": "string",
  "vendor": "string",
  "total": "float",
  "currency": "string"
}
```

## 🛠️ Data Generation Methodology

This is a **Synthetic Dataset** generated using a custom Python pipeline.

* **Variety:** Uses 20+ vendors, 20+ item categories, and 5 different currency types.
* **Robustness:** 7 different sentence templates were used to ensure the model doesn't overfit to a single way of speaking.
* **Formats:** Includes various date formats (e.g., "Oct 12", "12/10/23", "yesterday") to test the model's temporal reasoning.

## 💻 How to Use

You can load this dataset directly in your training script using the Hugging Face `datasets` library:

```python
from datasets import load_dataset

dataset = load_dataset("manuelaschrittwieser/invoice-extraction-dataset-v2")
print(dataset["train"][0])
```

## 🎯 Use Cases

* Fine-tuning Llama-3, Mistral, or Phi-3 for financial automation.
* Training "Parser Agents" for accounting software.
* Benchmarking small models on structured data tasks.

## 🏆 Project Context

This dataset was created as part of a **Structured Data Extractor** project. It serves as the training foundation for the `llama-3-invoice-extractor` model.

Developed by: [manuelaschrittwieser](https://huggingface.co/manuelaschrittwieser)



