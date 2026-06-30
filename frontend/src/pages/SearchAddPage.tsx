type ItemGroup = 'general' | 'academic' | 'paper';

type SearchAddPageProps = {
  group: ItemGroup;
};

export default function SearchAddPage({ group }: SearchAddPageProps) {
  return <div>SearchAddPage: {group}</div>;
}
