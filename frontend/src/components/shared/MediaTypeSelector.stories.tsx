import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { MediaTypeSelector } from "./MediaTypeSelector";
import type { MediaType } from "../../config/mediaTypes";

const meta: Meta<typeof MediaTypeSelector> = {
  title: "shared/MediaTypeSelector",
  component: MediaTypeSelector,
};

export default meta;
type Story = StoryObj<typeof MediaTypeSelector>;

function Interactive({
  initial,
  includeAll,
  compact,
}: {
  initial: MediaType | "all";
  includeAll?: boolean;
  compact?: boolean;
}) {
  const [value, setValue] = useState<MediaType | "all">(initial);
  return <MediaTypeSelector value={value} onChange={setValue} includeAll={includeAll} compact={compact} />;
}

export const Default: Story = {
  render: () => <Interactive initial="movie" />,
};

export const WithAllOption: Story = {
  render: () => <Interactive initial="all" includeAll />,
};

export const Compact: Story = {
  render: () => <Interactive initial="anime" compact />,
};
