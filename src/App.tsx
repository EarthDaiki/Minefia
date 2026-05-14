import { useState } from "react";
import "./App.css";

function App() {
  const [message, setMessage] = useState("");

  function handleClick() {
    setMessage("Hello from Minefia!");
  }

  return (
    <main className="container">
      <h1>Minefia</h1>

      <button onClick={handleClick}>
        Click me
      </button>

      <p>{message}</p>
    </main>
  );
}

export default App;