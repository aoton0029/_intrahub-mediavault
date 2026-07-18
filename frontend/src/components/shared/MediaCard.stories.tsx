import type { Meta, StoryObj } from "@storybook/react-vite";
import { MemoryRouter } from "react-router-dom";
import { MediaCard } from "./MediaCard";

const meta: Meta<typeof MediaCard> = {
  title: "shared/MediaCard",
  component: MediaCard,
  decorators: [
    (Story) => (
      <MemoryRouter>
        <div style={{ width: 200 }}>
          <Story />
        </div>
      </MemoryRouter>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof MediaCard>;

export const Default: Story = {
  args: {
    title: "進撃の巨人",
    badge: "TV",
    meta: "2013 / 25話",
    rating: 4.5,
    favorite: false,
    href: "#",
  },
};

export const Favorite: Story = {
  args: {
    ...Default.args,
    favorite: true,
  },
};

export const Compact: Story = {
  args: {
    ...Default.args,
    variant: "compact",
  },
};

export const SearchResult: Story = {
  args: {
    title: "呪術廻戦",
    badge: "TV",
    meta: "2020 / 24話",
    variant: "search-result",
    imported: false,
    actionLabel: "取り込む",
  },
};

export const SearchResultImported: Story = {
  args: {
    ...SearchResult.args,
    imported: true,
  },
};

export const Portrait: Story = {
  args: {
    ...Default.args,
    thumbnailOrientation: "vertical",
  },
};
