import type { Meta, StoryObj } from "@storybook/react-vite";
import { EmptyState } from "./EmptyState";

const meta: Meta<typeof EmptyState> = {
  title: "shared/EmptyState",
  component: EmptyState,
};

export default meta;
type Story = StoryObj<typeof EmptyState>;

export const Default: Story = {
  args: {
    title: "作品が見つかりません",
    description: "検索条件を変更するか、新しい作品を登録してください。",
  },
};

export const WithAction: Story = {
  args: {
    title: "まだ登録がありません",
    description: "最初の作品を登録して始めましょう。",
    action: <button className="btn btn-accent" type="button">作品を登録</button>,
  },
};
