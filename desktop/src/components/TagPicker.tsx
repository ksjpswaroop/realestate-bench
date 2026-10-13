/**
 * TagPicker — chip-style input for adding tags to an extraction.
 *
 * Free-form strings, normalized to lowercase + trim on the Rust side.
 * Shows the current tags as chips with an × to remove, plus an input
 * for new tags (Enter to add, Backspace on empty input to remove last).
 */
import { useEffect, useState } from "react";

type Props = {
  extractionId: number;
  initialTags: string[];
  onChange?: (tags: string[]) => void;
  /** Optional: existing tags to suggest from. */
  suggestions?: string[];
};

export default function TagPicker({ extractionId, initialTags, onChange, suggestions = [] }: Props) {
  const [tags, setTags] = useState<string[]>(initialTags);
  const [input, setInput] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setTags(initialTags);
  }, [extractionId, initialTags]);

  const add = async (raw: string) => {
    const t = raw.trim();
    if (!t) return;
    if (tags.includes(t.toLowerCase())) {
      setInput("");
      return;
    }
    try {
      const next = await (await import("../lib/ipc")).ipc.addExtractionTag(extractionId, t);
      setTags(next);
      setInput("");
      setError(null);
      onChange?.(next);
    } catch (e) {
      setError(`${e}`);
    }
  };

  const remove = async (tag: string) => {
    try {
      const next = await (await import("../lib/ipc")).ipc.removeExtractionTag(extractionId, tag);
      setTags(next);
      onChange?.(next);
    } catch (e) {
      setError(`${e}`);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      void add(input);
    } else if (e.key === "Backspace" && input === "" && tags.length > 0) {
      void remove(tags[tags.length - 1]);
    }
  };

  const suggestionSet = new Set(tags);
  const remainingSuggestions = suggestions.filter((s) => !suggestionSet.has(s));

  return (
    <div className="tag-picker" data-testid="tag-picker">
      <label className="tag-picker-label">Tags</label>
      <div className="tag-picker-chips">
        {tags.map((t) => (
          <span key={t} className="tag-chip">
            {t}
            <button
              type="button"
              className="tag-chip-remove"
              onClick={() => void remove(t)}
              aria-label={`Remove tag ${t}`}
            >×</button>
          </span>
        ))}
        <input
          type="text"
          className="tag-picker-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={onKeyDown}
          onBlur={() => input.trim() && void add(input)}
          placeholder={tags.length === 0 ? "Add a tag…" : ""}
          data-testid="tag-input"
        />
      </div>
      {error && <div className="tag-picker-error">{error}</div>}
      {remainingSuggestions.length > 0 && (
        <div className="tag-picker-suggestions">
          <span className="hint">Suggested:</span>
          {remainingSuggestions.slice(0, 8).map((s) => (
            <button
              key={s}
              type="button"
              className="tag-suggestion"
              onClick={() => void add(s)}
            >
              + {s}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
