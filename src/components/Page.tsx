import type { ReactNode } from "react";

export function Page({ title, description, children }: { title: string; description?: string; children?: ReactNode }) {
  return (
    <section className="mx-auto max-w-4xl p-8">
      <h1 className="text-2xl font-semibold">{title}</h1>
      {description && <p className="mt-1 text-neutral-600 dark:text-neutral-400">{description}</p>}
      <div className="mt-6">{children}</div>
    </section>
  );
}
