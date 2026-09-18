import { NavLink } from "react-router-dom";
import {
  Clock,
  FolderOpen,
  Home,
  Images,
  Radio,
  Settings,
  type LucideIcon,
} from "lucide-react";

import { cn } from "@/lib/utils";

type NavItem = {
  to: string;
  label: string;
  icon: LucideIcon;
};

const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Home", icon: Home },
  { to: "/live", label: "Live", icon: Radio },
  { to: "/bible", label: "Bible", icon: FolderOpen },
  { to: "/media", label: "Media", icon: Images },
  { to: "/presentations", label: "Presentations", icon: Clock },
  { to: "/settings", label: "Settings", icon: Settings },
];

/**
 * Operator navigation rail. Collapses to icons on narrow windows so the Live
 * screen keeps the width it needs during a service.
 */
export default function Sidebar() {
  return (
    <nav className="flex h-full w-16 shrink-0 flex-col border-r border-border/60 bg-sidebar text-sidebar-foreground lg:w-56">
      <div className="flex h-14 items-center gap-2.5 px-3 lg:px-5">
        <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-primary text-sm font-bold text-primary-foreground">
          S
        </span>
        <span className="hidden text-sm font-semibold tracking-[0.22em] lg:inline">
          SELAH
        </span>
      </div>

      <ul className="flex flex-1 flex-col gap-0.5 px-2 py-2">
        {NAV_ITEMS.map((item) => (
          <li key={item.to}>
            <NavLink
              to={item.to}
              end={item.to === "/"}
              title={item.label}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  "text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                  isActive &&
                    "bg-sidebar-accent text-sidebar-foreground shadow-sm ring-1 ring-inset ring-border/60",
                )
              }
            >
              <item.icon className="size-4 shrink-0" />
              <span className="hidden lg:inline">{item.label}</span>
            </NavLink>
          </li>
        ))}
      </ul>

      <div className="hidden px-5 py-4 text-[10px] uppercase tracking-[0.18em] text-muted-foreground/70 lg:block">
        offline · local-first
      </div>
    </nav>
  );
}
