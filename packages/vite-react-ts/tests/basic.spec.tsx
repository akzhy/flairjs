import { cleanup, render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { afterEach, expect, test } from "vitest";
import { TestCaseComponent } from "./component";

afterEach(() => {
  cleanup();
});

test("case-1", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case1");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("red");
  expect(element).toHaveStyle({ color: "rgb(255, 0, 1)" });
});

test("case-2", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case2");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-2 case-2-1");
  expect(element).toHaveClass("case-2-2");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 2)",
    backgroundColor: "rgb(255, 0, 2)",
  });
});

test("case-3", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case3");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-3");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 3)",
  });
});

test("case-4", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case4");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-4");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 4)",
  });
});

test("case-5", async () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case5");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-5 case-5-1 case-5-2");
  expect(element).toHaveClass("case-5-3");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 5)",
    backgroundColor: "rgb(255, 0, 5)",
  });

  await userEvent.click(element);

  expect(element).toHaveStyle({
    backgroundColor: "rgb(255, 0, 6)",
  });
});

test("case-6", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case6");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-6 case-6-1 case-6-3 case-6-4");
  expect(element).toHaveClass("case-6-2");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 6)",
    backgroundColor: "rgb(255, 0, 6)",
    borderColor: "rgb(255, 0, 6)",
    outlineColor: "rgb(255, 0, 6)",
  });
});

test("case-7", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case7");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-7-1");
  expect(element).toHaveClass("case-7");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 7)",
  });
});

test("case-8", async () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case8");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-8-1 case-8-2");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 8)",
  });
  await userEvent.click(element);
  expect(element).toHaveStyle({
    backgroundColor: "rgb(255, 0, 8)",
  });
});

test("case-9", async () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case9");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-9-1 case-9-2");
  expect(element).toHaveClass("case-9");

  expect(element).toHaveStyle({
    backgroundColor: "rgb(255, 0, 9)",
  });
  await userEvent.click(element);
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 9)",
  });
});

test("case-10", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case10");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-10");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 10)",
  });
});

test("case-11", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case11");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-11-1");
  expect(element).toHaveClass("case-11 case-11-2");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 11)",
  });
});

test("case-12", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case12");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-12-1");
  expect(element).toHaveClass("case-12 case-12-2");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 12)",
  });
});

test("case-13", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case13");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-13");
  expect(element).toHaveClass("case-13-1");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 13)",
  });
});

test("case-14", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case14");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-14");
  expect(element).toHaveClass("case-14-1 case-14-2");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 14)",
  });
});

test("case-15", () => {
  render(<TestCaseComponent />);

  const element = screen.getByText("Case15");
  expect(element).toBeInTheDocument();
  expect(element).not.toHaveClass("case-15");
  expect(element).toHaveClass("case-15-1");
  expect(element).toHaveStyle({
    color: "rgb(255, 0, 15)",
  });
});
