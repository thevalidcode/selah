import type { ReactNode } from "react";

import { cn } from "@/lib/utils";
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

/**
 * Standard page header: title, optional subtitle and right-aligned actions.
 * Used by every operator screen so the layout stays consistent.
 */
export default function PageHeader({
  title,
  subtitle,
  actions,
  className,
}: {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <header
      className={cn(
        "flex flex-wrap items-start justify-between gap-3 border-b border-border/60 pb-4",
        className,
      )}
    >
      <div className="min-w-0">
        <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
        {subtitle ? (
          <p className="mt-0.5 text-sm text-muted-foreground">{subtitle}</p>
        ) : null}
      </div>
      {actions ? (
        <div className="flex shrink-0 items-center gap-2">{actions}</div>
      ) : null}
    </header>
  );
}

/**
 * Titled content panel. Thin wrapper around shadcn's Card so panels stay
 * visually identical across screens.
 */
export function Panel({
  title,
  actions,
  children,
  className,
  contentClassName,
}: {
  title?: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}) {
  return (
    <Card className={className}>
      {title ? (
        <CardHeader>
          <CardTitle>{title}</CardTitle>
          {actions ? <CardAction>{actions}</CardAction> : null}
        </CardHeader>
      ) : null}
      <CardContent className={contentClassName}>{children}</CardContent>
    </Card>
  );
}

/** Small status pill with a pulsing dot (listening / idle / error). */
export function StatusPill({
  active,
  label,
  tone = "neutral",
}: {
  active: boolean;
  label: string;
  tone?: "neutral" | "success" | "destructive" | "warning";
}) {
  const dot =
    tone === "success" || (tone === "neutral" && active)
      ? "bg-success"
      : tone === "destructive"
        ? "bg-destructive"
        : tone === "warning"
          ? "bg-warning"
          : "bg-muted-foreground";

  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border border-border/70 bg-card px-3 py-1 text-xs font-medium",
        active && "border-success/40",
      )}
    >
      <span className="relative flex size-2">
        {active ? (
          <span
            className={cn(
              "absolute inline-flex size-full animate-ping rounded-full opacity-60",
              dot,
            )}
          />
        ) : null}
        <span className={cn("relative inline-flex size-2 rounded-full", dot)} />
      </span>
      {label}
    </span>
  );
}

/** Muted empty-state hint. */
export function EmptyHint({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <p className={cn("text-sm text-muted-foreground", className)}>{children}</p>
  );
}

/** Definition list used for compact key/value readouts. */
export function KeyValueList({
  items,
  className,
}: {
  items: { label: string; value: ReactNode }[];
  className?: string;
}) {
  return (
    <dl className={cn("divide-y divide-border/50", className)}>
      {items.map((item) => (
        <div
          key={item.label}
          className="flex items-baseline justify-between gap-4 py-2 first:pt-0 last:pb-0"
        >
          <dt className="text-xs uppercase tracking-[0.1em] text-muted-foreground">
            {item.label}
          </dt>
          <dd className="selectable max-w-[60%] truncate text-right text-sm font-medium">
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}
