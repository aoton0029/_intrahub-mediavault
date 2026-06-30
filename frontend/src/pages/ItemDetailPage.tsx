import { useParams } from 'react-router-dom';

export default function ItemDetailPage() {
  const { id } = useParams<{ id: string }>();
  return <div>ItemDetailPage: {id}</div>;
}
