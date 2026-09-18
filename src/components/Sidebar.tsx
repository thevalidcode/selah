import { useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import {
  FolderOpen,
  Home,
  Images,
  Presentation,
  Radio,
  Settings,
  type LucideIcon,
} from "lucide-react";

import { SelahIcon } from "@/components/ui/selah-icon";
import { cn } from "@/lib/utils";

type NavItem = {
  to: string;
  label: string;
  icon: LucideIcon;
};

type NavGroup = {
  /** Small-caps heading. Omitted for the first group so it sits flush. */
  label?: string;
  items: NavItem[];
};

/**
 * Grouped like a desktop application rather than a flat website menu: the two
 * screens an operator touches during a service (Home, Live) are separated from
 * the content library.
 */
const GROUPS: NavGroup[] = [
  {
    items: [
      { to: "/", label: "Home", icon: Home },
      { to: "/live", label: "Live", icon: Radio },
    ],
  },
  {
    label: "Library",
    items: [
      { to: "/bible", label: "Bible", icon: FolderOpen },
      { to: "/media", label: "Media", icon: Images },
      { to: "/presentations", label: "Presentations", icon: Presentation },
    ],
  },
];

const SETTINGS_ITEM: NavItem = {
  to: "/settings",
  label: "Settings",
  icon: Settings,
};

/** True while the rail is narrow enough that labels must be dropped. */
function useNarrowRail(): boolean {
  const [narrow, setNarrow] = useState(false);

  useEffect(() => {
    // 1024px matches Tailwind's `lg`. Kept in sync by hand because the rail
    // width and the label visibility must change at exactly the same point.
    const query = window.matchMedia("(max-width: 1023px)");
    const sync = () => setNarrow(query.matches);
    sync();
    query.addEventListener("change", sync);
    return () => query.removeEventListener("change", sync);
  }, []);

  return narrow;
}

/**
 * Operator navigation rail.
 *
 * Built to read as a native panel rather than a web page: a flat full-height
 * surface with its own identity header, dense 32px rows, and Settings pinned to
 * the foot. It stays deliberately narrow — the Live screen is what needs room.
 */
export default function Sidebar() {
  const narrow = useNarrowRail();

  return (
    <nav
      aria-label="Main"
      className={cn(
        "flex h-full shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground",
        narrow ? "w-14" : "w-52",
      )}
    >
      {/* Identity header — the strip every desktop app has at the top. */}
      <div
        className={cn(
          "flex h-12 shrink-0 items-center border-b border-sidebar-border",
          narrow ? "justify-center" : "px-3",
        )}
      >
        <span className="flex items-center gap-2.5 overflow-hidden">
          <SelahIcon size={26} tile title="Selah" className="rounded-[7px]" />
          {!narrow ? (
            <span className="text-[0.8125rem] font-black tracking-[0.2em]">
              SELAH
            </span>
          ) : null}
        </span>
      </div>

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto px-2 py-3">
        {GROUPS.map((group) => (
          <div key={group.label ?? "primary"} className="flex flex-col gap-0.5">
            {group.label && !narrow ? (
              <p className="px-2.5 pb-1 text-[0.625rem] font-bold tracking-[0.14em] text-muted-foreground/60 uppercase">
                {group.label}
              </p>
            ) : null}
            {group.items.map((item) => (
              <RailLink key={item.to} item={item} narrow={narrow} />
            ))}
          </div>
        ))}
      </div>

      {/* Settings sits apart from the working set. */}
      <div className="shrink-0 border-t border-sidebar-border px-2 py-2">
        <RailLink item={SETTINGS_ITEM} narrow={narrow} />
        {!narrow ? (
          <p className="px-2.5 pt-1.5 text-[0.6875rem] leading-snug text-muted-foreground/70">
            Works without internet
          </p>
        ) : null}
      </div>
    </nav>
  );
}

/**
 * One rail row.
 *
 * Uses the native `title` attribute rather than a custom tooltip so the label
 * is drawn by the operating system, the way a real desktop sidebar behaves.
 */
function RailLink({ item, narrow }: { item: NavItem; narrow: boolean }) {
  return (
    <NavLink
      to={item.to}
      end={item.to === "/"}
      title={narrow ? item.label : undefined}
      className={({ isActive }) =>
        cn(
          "relative flex h-8 items-center rounded-md text-[0.8125rem] font-medium transition-colors",
          narrow ? "justify-center px-0" : "gap-2.5 px-2.5",
          "text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
          isActive && "bg-brand/12 text-brand hover:bg-brand/15",
        )
      }
    >
      {({ isActive }) => (
        <>
          {isActive ? (
            <span className="absolute top-1/2 left-0 h-4 w-0.5 -translate-y-1/2 rounded-full bg-brand" />
          ) : null}
          <item.icon className="size-4 shrink-0" />
          {!narrow ? <span className="truncate">{item.label}</span> : null}
        </>
      )}
    </NavLink>
  );
}
