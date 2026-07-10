import { Link } from "react-router-dom";

type SectionHeadingProps = {
  title: string;
  seeAllHref: string;
};

export function SectionHeading({ title, seeAllHref }: SectionHeadingProps) {
  return (
    <div className="section-heading">
      <span>{title}</span>
      <Link className="see-all" to={seeAllHref}>
        すべて見る →
      </Link>
    </div>
  );
}
