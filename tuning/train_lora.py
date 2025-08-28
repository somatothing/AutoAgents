import argparse
import json
import os
from dataclasses import dataclass

import torch
from datasets import load_dataset
from transformers import AutoModelForCausalLM, AutoTokenizer, Trainer, TrainingArguments
from peft import LoraConfig, get_peft_model


@dataclass
class DataArgs:
    field_prompt: str = "prompt"
    field_response: str = "response"


def load_jsonl(path):
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                yield json.loads(line)


def format_example(ex, da: DataArgs):
    return f"<user>\n{ex[da.field_prompt]}\n</user>\n<assistant>\n{ex[da.field_response]}\n</assistant>"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--model', type=str, default=os.environ.get('QWEN_MODEL', 'Qwen/Qwen2.5-Coder-7B'))
    parser.add_argument('--data', type=str, required=True, help='path to jsonl data')
    parser.add_argument('--output', type=str, default='./outputs/qwen_lora')
    parser.add_argument('--lr', type=float, default=1e-4)
    parser.add_argument('--batch', type=int, default=1)
    parser.add_argument('--epochs', type=int, default=1)
    parser.add_argument('--bf16', action='store_true')
    args = parser.parse_args()

    device_map = 'auto'
    torch_dtype = torch.bfloat16 if args.bf16 else torch.float16

    tokenizer = AutoTokenizer.from_pretrained(args.model, trust_remote_code=True)
    model = AutoModelForCausalLM.from_pretrained(args.model, trust_remote_code=True, torch_dtype=torch_dtype, device_map=device_map)

    peft_cfg = LoraConfig(r=16, lora_alpha=32, lora_dropout=0.05, target_modules=["q_proj", "v_proj"])  # adjust as needed
    model = get_peft_model(model, peft_cfg)

    data_args = DataArgs()
    data_list = list(load_jsonl(args.data))

    def preprocess_fn(batch):
        texts = [format_example(ex, data_args) for ex in batch]
        toks = tokenizer(texts, padding=True, truncation=True, max_length=4096, return_tensors='pt')
        toks["labels"] = toks["input_ids"].clone()
        return toks

    class SimpleDataset(torch.utils.data.Dataset):
        def __init__(self, data):
            self.data = data
        def __len__(self):
            return len(self.data)
        def __getitem__(self, idx):
            return self.data[idx]

    processed = [preprocess_fn([ex]) for ex in data_list]
    dataset = SimpleDataset(processed)

    def collate(batch):
        input_ids = torch.nn.utils.rnn.pad_sequence([b["input_ids"].squeeze(0) for b in batch], batch_first=True, padding_value=tokenizer.pad_token_id)
        attention_mask = torch.nn.utils.rnn.pad_sequence([b["attention_mask"].squeeze(0) for b in batch], batch_first=True, padding_value=0)
        labels = torch.nn.utils.rnn.pad_sequence([b["labels"].squeeze(0) for b in batch], batch_first=True, padding_value=-100)
        return {"input_ids": input_ids, "attention_mask": attention_mask, "labels": labels}

    train_args = TrainingArguments(
        output_dir=args.output,
        learning_rate=args.lr,
        num_train_epochs=args.epochs,
        per_device_train_batch_size=args.batch,
        gradient_accumulation_steps=8,
        logging_steps=10,
        save_steps=200,
        save_total_limit=2,
        bf16=args.bf16,
        optim='adamw_torch',
    )

    trainer = Trainer(model=model, args=train_args, train_dataset=dataset, data_collator=collate)
    trainer.train()
    trainer.save_model(args.output)


if __name__ == '__main__':
    main()

