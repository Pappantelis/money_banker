import type { Transaction, Category } from "../types";

interface TransactionListProps {
  transactions: Transaction[];
  categories: Category[];
}

export function TransactionList({ transactions, categories }: TransactionListProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("el-GR", {
      style: "currency",
      currency: "EUR",
    }).format(Math.abs(amount));
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString("el-GR", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    });
  };

  const getCategoryName = (categoryId?: string) => {
    if (!categoryId) return "—";
    const category = categories.find((c) => c.id === categoryId);
    return category?.name ?? "—";
  };

  const getCategoryColor = (categoryId?: string) => {
    if (!categoryId) return "#888";
    const category = categories.find((c) => c.id === categoryId);
    return category?.color ?? "#888";
  };

  if (transactions.length === 0) {
    return (
      <div className="transaction-list empty">
        <p>No transactions for this month</p>
      </div>
    );
  }

  return (
    <div className="transaction-list">
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Store / Description</th>
            <th>Category</th>
            <th>Source</th>
            <th className="amount-col">Amount</th>
          </tr>
        </thead>
        <tbody>
          {transactions.map((tx) => (
            <tr key={tx.id}>
              <td className="date-cell">{formatDate(tx.transaction_date)}</td>
              <td className="store-cell">{tx.store || tx.description || "—"}</td>
              <td className="category-cell">
                <span
                  className="category-badge"
                  style={{ backgroundColor: getCategoryColor(tx.category_id) }}
                >
                  {getCategoryName(tx.category_id)}
                </span>
              </td>
              <td className="source-cell">
                <span className={`source-badge ${tx.source}`}>
                  {tx.source === "email" ? "📧" : "✏️"}
                </span>
              </td>
              <td className={`amount-cell ${tx.amount >= 0 ? "income" : "expense"}`}>
                {tx.amount >= 0 ? "+" : "-"}
                {formatCurrency(tx.amount)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
