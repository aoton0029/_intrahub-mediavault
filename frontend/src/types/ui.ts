import type { IconType } from 'react-icons';
import type { ReactNode } from 'react';

export type Theme = 'dark' | 'light';
export type ItemStatus = 'not_started' | 'in_progress' | 'done';

export interface BreadcrumbItem {
  label: string;
  to?: string;
}

export interface AppRouteHandle {
  title: string;
  breadcrumb: BreadcrumbItem[];
  actions?: ReactNode;
}

export interface NavItemConfig {
  label: string;
  to: string;
  count?: number;
  indent?: boolean;
  icon: IconType;
  match?: (pathname: string) => boolean;
}

export interface NavSectionConfig {
  label: string;
  items: NavItemConfig[];
}

export interface TagLikeItem {
  id: string;
  label: string;
}

export interface MediaCardItem {
  id: string;
  title: string;
  meta: string;
  badge?: string;
  isFavorite?: boolean;
  rating?: number;
  coverUrl?: string;
  to?: string;
  actionLabel?: string;
  actionDisabled?: boolean;
}

export interface LiteratureItem {
  id: string;
  title: string;
  byline: string;
  meta?: string;
  doi?: string;
  tags?: TagLikeItem[];
  favorite?: boolean;
  rating?: number;
  coverUrl?: string;
  to?: string;
}

export interface FilterChip {
  id: string;
  label: string;
  active?: boolean;
  addTrigger?: boolean;
}

export interface SelectOption {
  value: string;
  label: string;
}

export interface ResourceTabItem {
  key: string;
  label: string;
  icon: IconType;
  content: ReactNode;
}

export interface RailMetaItem {
  id: string;
  label: string;
  value: string;
  muted?: boolean;
  icon: IconType;
}

export interface DetailMylistItem {
  id: string;
  label: string;
}

export interface PropertyItem {
  key: string;
  label: string;
  value: ReactNode;
  muted?: boolean;
}

export interface EpisodeItem {
  id: string;
  number: string;
  title: string;
  meta?: string;
}

export interface GroupItem {
  id: string;
  title: string;
  subtitle?: string;
  episodes: EpisodeItem[];
}

export interface StaffMember {
  id: string;
  label: string;
  subLabel?: string;
}

export interface StreamingLinkItem {
  id: string;
  label: string;
  subLabel?: string;
}

export interface RelatedWorkItem {
  id: string;
  title: string;
  meta: string;
  to?: string;
}
