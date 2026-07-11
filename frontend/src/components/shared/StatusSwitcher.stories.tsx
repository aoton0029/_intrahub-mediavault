import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { StatusSwitcher, type StatusValue } from "./StatusSwitcher";

const meta: Meta<typeof StatusSwitcher> = {
  title: "shared/StatusSwitcher",
  component: StatusSwitcher,
};

export default meta;
type Story = StoryObj<typeof StatusSwitcher>;

function Interactive({ initial }: { initial: StatusValue }) {
  const [value, setValue] = useState<StatusValue>(initial);
  return <StatusSwitcher value={value} onChange={setValue} />;
}

export const NotStarted: Story = {
  render: () => <Interactive initial="not_started" />,
};

export const InProgress: Story = {
  render: () => <Interactive initial="in_progress" />,
};

export const Completed: Story = {
  render: () => <Interactive initial="completed" />,
};
