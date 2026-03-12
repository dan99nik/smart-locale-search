import type { ReactNode } from "react";

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return <div className="flex flex-col h-screen bg-bg-primary">{children}</div>;
}
