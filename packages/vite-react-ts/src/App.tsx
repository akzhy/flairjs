import { useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";
import { Style } from "jsx-styled-react";
import clsx from "clsx";

const cardClassName = "card";

function App() {
  const [count, setCount] = useState(0);

  return (
    <>
      <div>
        <a
          href="https://vite.dev"
          target="_blank"
          className={count % 2 === 0 && count > 2 && "link-primary"}
        >
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a
          href="https://react.dev"
          target="_blank"
          className={count % 2 === 0 ? "link-primary" : "link-secondary"}
        >
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className={cardClassName}>
        <button
          onClick={() => setCount((count) => count + 1)}
          className={clsx(
            "btn",
            {
              "btn-primary": count % 2 === 0,
              "btn-secondary": count % 2 === 1,
            },
            ["btn-large", "btn-outline"]
          )}
        >
          count is {count}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
      <Style>
        {
          /* css */ `
          .read-the-docs {
            padding: 2em;
            color: green;
          }

          .btn {
            background-color: #3c49db;
          }

          .card {
            background-color: #3c49db;
          }

          .btn-primary {
            background-color: #db793c;
          }

          .btn-secondary {
            background-color: #bb2be7;
          }

          .btn-primary:hover {
            background-color: #b45419;
          }

          .btn-large {
            padding: 1rem 2rem;
          }

          .btn-outline {
            border: 2px solid currentColor;
          }

          .link-primary {
            border: 2px solid #3c49db;
          }

          .link-secondary {
            border: 2px solid #bb2be7;
          }
        `
        }
      </Style>
    </>
  );
}

export default App;
