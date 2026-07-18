import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { MediaTypeDropdown } from "./MediaTypeDropdown";
import type { MediaType } from "../../config/mediaTypes";

const meta: Meta<typeof MediaTypeDropdown> = {
  title: "shared/MediaTypeDropdown",
  component: MediaTypeDropdown,
};

export default meta;
type Story = StoryObj<typeof MediaTypeDropdown>;

function Interactive({ initial, includeAll }: { initial: MediaType | "all"; includeAll?: boolean }) {
  const [value, setValue] = useState<MediaType | "all">(initial);
  return <MediaTypeDropdown value={value} onChange={setValue} includeAll={includeAll} />;
}

export const Default: Story = {
  render: () => <Interactive initial="movie" />,
};

export const WithAllOption: Story = {
  render: () => <Interactive initial="all" includeAll />,
};
