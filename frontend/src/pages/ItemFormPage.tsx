import { useParams } from 'react-router-dom';

type ItemGroup = 'general' | 'academic' | 'paper';
type ItemFormMode = 'create' | 'edit';

type ItemFormPageProps = {
  mode: ItemFormMode;
  group?: ItemGroup;
};

export default function ItemFormPage({ mode, group }: ItemFormPageProps) {
  const { id } = useParams<{ id: string }>();
  return (
    <div>
      ItemFormPage: {mode} {group ?? ''} {id ?? ''}
    </div>
  );
}
