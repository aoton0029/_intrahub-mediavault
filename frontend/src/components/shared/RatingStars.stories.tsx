import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { RatingStars } from "./RatingStars";

const meta: Meta<typeof RatingStars> = {
  title: "shared/RatingStars",
  component: RatingStars,
};

export default meta;
type Story = StoryObj<typeof RatingStars>;

export const ReadOnly: Story = {
  args: {
    value: 3.5,
    readOnly: true,
  },
};

export const Empty: Story = {
  args: {
    value: 0,
    readOnly: true,
  },
};

export const Interactive: Story = {
  render: () => {
    function Wrapper() {
      const [value, setValue] = useState(2);
      return <RatingStars value={value} onChange={setValue} />;
    }
    return <Wrapper />;
  },
};
