import ProductCardDeck from "@/components/ProductCardDeck";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-sand-200 dark:bg-sand-950">
      <div className="absolute top-4 right-4 z-50">
        <Link href="/login">
          <Button variant="outline" className="border-sand-300 dark:border-sand-800 text-sand-900 dark:text-sand-50">
            Sign In
          </Button>
        </Link>
      </div>
      <div className="w-full flex-1 flex flex-col items-center justify-center">
        <ProductCardDeck />
      </div>
    </main>
  );
}
