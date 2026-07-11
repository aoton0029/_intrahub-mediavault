import { useContext, useEffect } from "react";
import { PageChromeContext, type PageChrome } from "./pageChromeContext";

export function usePageChrome(value: PageChrome | null) {
  const context = useContext(PageChromeContext);

  useEffect(() => {
    context?.setPageChrome(value);

    return () => {
      context?.setPageChrome(null);
    };
  }, [context, value]);
}
