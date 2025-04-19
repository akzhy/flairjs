import * as t from "@babel/types";

export const extractCSS = (node: t.JSXElement) => {
  const styleBody = node.children
    .map((child) => {
      if (child.type === "JSXText") {
        return child.value;
      }
      if (child.type === "JSXExpressionContainer") {
        if (child.expression.type === "TemplateLiteral") {
          return child.expression.quasis.map((q) => q.value.raw).join("");
        }
      }

      return "";
    })
    .join("\n");

  return styleBody;
};
