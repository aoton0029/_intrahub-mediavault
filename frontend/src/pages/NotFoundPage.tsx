import { Link } from 'react-router-dom';
import { Button } from '@/components/ui/button';

export default function NotFoundPage() {
  return (
    <div className="flex flex-col items-center justify-center gap-6 p-16 text-center">
      <h1 className="text-6xl font-bold text-muted-foreground">404</h1>
      <p className="text-lg">ページが見つかりませんでした</p>
      <Button asChild>
        <Link to="/">一覧へ戻る</Link>
      </Button>
    </div>
  );
}
