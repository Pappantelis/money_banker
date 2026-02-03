import { useState } from "react";
import type { Category, CreateTransaction } from "../types";

interface AddTransactionProps {
  categories: Category[];
  onSubmit: (tx: CreateTransaction) => Promise<void>;
  onClose: () => void;
}

export function AddTransaction({ categories, onSubmit, onClose }: AddTransactionProps) {
  const [isIncome, setIsIncome] = useState(false);
  const [amount, setAmount] = useState("");
  const [store, setStore] = useState("");
  const [description, setDescription] = useState("");
  const [categoryId, setCategoryId] = useState("");
  const [date, setDate] = useState(() => {
    const today = new Date();
    return today.toISOString().split("T")[0];
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState("");

  const filteredCategories = categories.filter((c) => c.is_income === isIncome);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    const parsedAmount = parseFloat(amount);
    if (isNaN(parsedAmount) || parsedAmount <= 0) {
      setError("Please enter a valid amount");
      return;
    }

    try {
      setIsSubmitting(true);
      await onSubmit({
        amount: isIncome ? parsedAmount : -parsedAmount,
        store: store || undefined,
        description: description || undefined,
        category_id: categoryId || undefined,
        transaction_date: date,
        is_income: isIncome,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add transaction");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Add Transaction</h2>
          <button className="close-button" onClick={onClose}>×</button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="form-group type-toggle">
            <button
              type="button"
              className={`toggle-button ${!isIncome ? "active" : ""}`}
              onClick={() => {
                setIsIncome(false);
                setCategoryId("");
              }}
            >
              Expense
            </button>
            <button
              type="button"
              className={`toggle-button ${isIncome ? "active" : ""}`}
              onClick={() => {
                setIsIncome(true);
                setCategoryId("");
              }}
            >
              Income
            </button>
          </div>

          <div className="form-group">
            <label htmlFor="amount">Amount (€)</label>
            <input
              type="number"
              id="amount"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              step="0.01"
              min="0"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="store">Store / Source</label>
            <input
              type="text"
              id="store"
              value={store}
              onChange={(e) => setStore(e.target.value)}
              placeholder="e.g., Lidl, Shell, Netflix"
            />
          </div>

          <div className="form-group">
            <label htmlFor="description">Description (optional)</label>
            <input
              type="text"
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Additional notes"
            />
          </div>

          <div className="form-group">
            <label htmlFor="category">Category</label>
            <select
              id="category"
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value)}
            >
              <option value="">Select category</option>
              {filteredCategories.map((cat) => (
                <option key={cat.id} value={cat.id}>
                  {cat.icon && `${cat.icon} `}{cat.name}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="date">Date</label>
            <input
              type="date"
              id="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
              required
            />
          </div>

          {error && <div className="error-message">{error}</div>}

          <div className="form-actions">
            <button type="button" className="cancel-button" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="submit-button" disabled={isSubmitting}>
              {isSubmitting ? "Adding..." : "Add Transaction"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
