interface SnippetPreviewProps {
  snippet: string;
  query?: string;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function highlightQuery(snippet: string, query: string | undefined): string {
  if (snippet.includes("<mark>")) return snippet;

  if (!query || !query.trim()) return escapeHtml(snippet);

  const terms = query
    .trim()
    .split(/\s+/)
    .filter((t) => t.length >= 2)
    .map(escapeRegex);

  if (terms.length === 0) return escapeHtml(snippet);

  const pattern = new RegExp(`(${terms.join("|")})`, "gi");

  const parts: string[] = [];
  let lastIndex = 0;

  for (const match of snippet.matchAll(pattern)) {
    const start = match.index!;
    const end = start + match[0].length;
    parts.push(escapeHtml(snippet.slice(lastIndex, start)));
    parts.push(`<mark>${escapeHtml(match[0])}</mark>`);
    lastIndex = end;
  }
  parts.push(escapeHtml(snippet.slice(lastIndex)));

  return parts.join("");
}

export function SnippetPreview({ snippet, query }: SnippetPreviewProps) {
  const html = highlightQuery(snippet, query);
  return (
    <p
      className="text-sm text-text-secondary leading-relaxed line-clamp-2 mt-1"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
