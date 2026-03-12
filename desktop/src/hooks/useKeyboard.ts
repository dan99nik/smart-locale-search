import { useState, useCallback, useEffect } from "react";

export function useKeyboard(resultCount: number) {
  const [selectedIndex, setSelectedIndex] = useState(-1);

  useEffect(() => {
    setSelectedIndex(-1);
  }, [resultCount]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) =>
            prev < resultCount - 1 ? prev + 1 : prev
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => (prev > 0 ? prev - 1 : -1));
          break;
        case "Escape":
          setSelectedIndex(-1);
          break;
      }
    },
    [resultCount]
  );

  return { selectedIndex, setSelectedIndex, handleKeyDown };
}
