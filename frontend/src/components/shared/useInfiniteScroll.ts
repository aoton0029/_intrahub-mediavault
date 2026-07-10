import { useEffect, useRef } from "react";

export function useInfiniteScroll(onLoadMore: () => void, enabled = true) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!enabled || !ref.current) return;

    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        onLoadMore();
      }
    });

    observer.observe(ref.current);
    return () => observer.disconnect();
  }, [enabled, onLoadMore]);

  return ref;
}
