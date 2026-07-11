import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { TagList, type TagListItem } from "./TagList";

const meta: Meta<typeof TagList> = {
  title: "shared/TagList",
  component: TagList,
};

export default meta;
type Story = StoryObj<typeof TagList>;

const initialTags: TagListItem[] = [
  { id: "1", name: "SF" },
  { id: "2", name: "感動" },
];

function Interactive({ kind }: { kind: "tag" | "category" }) {
  const [items, setItems] = useState<TagListItem[]>(initialTags);
  return (
    <TagList
      kind={kind}
      items={items}
      onAdd={(name) => setItems((current) => [...current, { id: String(Date.now()), name }])}
      onRemove={(id) => setItems((current) => current.filter((item) => item.id !== id))}
    />
  );
}

export const Tags: Story = {
  render: () => <Interactive kind="tag" />,
};

export const Categories: Story = {
  render: () => <Interactive kind="category" />,
};

export const Empty: Story = {
  args: {
    kind: "tag",
    items: [],
  },
};
