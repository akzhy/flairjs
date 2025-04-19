import * as t from "@babel/types";
import { CSSModuleExports } from "lightningcss";

export const processAttribute = ({
  node,
  attrName,
  classNameMap,
}: {
  node: t.JSXAttribute;
  attrName: string;
  classNameMap: CSSModuleExports;
}) => {
  if (node.value?.type === "StringLiteral") {
    const transformedClassNames = processStringLiteral(
      node.value,
      classNameMap
    );
    node.value = t.stringLiteral(transformedClassNames);
  } else if (node.value?.type === "JSXExpressionContainer") {
    if (node.value.expression.type === "CallExpression") {
      const transformedArgs = processCallExpression(
        node.value.expression,
        classNameMap
      );
      node.value.expression.arguments = transformedArgs;
    } else if (node.value.expression.type === "LogicalExpression") {
      const transformedNode = processLogicalExpression(
        node.value.expression,
        classNameMap
      );
      node.value.expression = transformedNode;
    } else if (node.value.expression.type === "ConditionalExpression") {
      const transformedNode = processConditionalExpression(
        node.value.expression,
        classNameMap
      );
      node.value.expression = transformedNode;
    }
  }
};

const processStringLiteral = (
  node: t.StringLiteral,
  classNameMap: CSSModuleExports
) => {
  const classNames = node.value.split(" ");
  const transformedClassNames = classNames.map((className) => {
    const exportName = classNameMap[className]?.name;
    return exportName ?? className;
  });

  return transformedClassNames.join(" ");
};

const processObjectExpression = (
  node: t.ObjectExpression,
  classNameMap: CSSModuleExports
) => {
  const properties = node.properties.map((property) => {
    if (property.type === "ObjectProperty" && t.isStringLiteral(property.key)) {
      const exportName = classNameMap[property.key.value];
      if (exportName) {
        return t.objectProperty(
          t.stringLiteral(exportName.name),
          property.value
        );
      }
    }
    return property;
  });
  return properties;
};

const processArrayExpression = (
  node: t.ArrayExpression,
  classNameMap: CSSModuleExports
) => {
  const elements: t.ArrayExpression["elements"] = node.elements.map(
    (element) => {
      switch (element?.type) {
        case "ObjectExpression":
          const properties = processObjectExpression(element, classNameMap);
          element.properties = properties;
          return element;
        case "StringLiteral":
          const value = processStringLiteral(element, classNameMap);
          element.value = value;
          return element;
        case "ArrayExpression":
          const elements = processArrayExpression(element, classNameMap);
          element.elements = elements;
          return element;
        case "LogicalExpression":
          return processLogicalExpression(element, classNameMap);
        case "ConditionalExpression":
          return processConditionalExpression(element, classNameMap);
        default:
          return element;
      }
    }
  );
  return elements;
};

const processLogicalExpression = (
  node: t.LogicalExpression,
  classNameMap: CSSModuleExports
) => {
  switch (node.right.type) {
    case "StringLiteral":
      const transformedClassNames = processStringLiteral(
        node.right,
        classNameMap
      );
      node.right.value = transformedClassNames;
      return node;
    case "ObjectExpression":
      const properties = processObjectExpression(node.right, classNameMap);
      node.right.properties = properties;
      return node;
    case "ArrayExpression":
      const elements = processArrayExpression(node.right, classNameMap);
      node.right.elements = elements;
      return node;
    default:
      return node;
  }
};

const processConditionalExpression = (
  node: t.ConditionalExpression,
  classNameMap: CSSModuleExports
) => {
  switch (node.consequent.type) {
    case "StringLiteral":
      const transformedClassNames = processStringLiteral(
        node.consequent,
        classNameMap
      );
      node.consequent.value = transformedClassNames;
      return node;
    case "ObjectExpression":
      const properties = processObjectExpression(node.consequent, classNameMap);
      node.consequent.properties = properties;
      return node;
    case "ArrayExpression":
      const elements = processArrayExpression(node.consequent, classNameMap);
      node.consequent.elements = elements;
      return node;
    default:
      return node;
  }
};

const processCallExpression = (
  node: t.CallExpression,
  classNameMap: CSSModuleExports
) => {
  const args = node.arguments.map((arg) => {
    if (arg.type === "StringLiteral") {
      const transformedClassNames = processStringLiteral(arg, classNameMap);
      arg.value = transformedClassNames;
      return arg;
    } else if (arg.type === "ObjectExpression") {
      const properties = processObjectExpression(arg, classNameMap);
      arg.properties = properties;
    } else if (arg.type === "ArrayExpression") {
      const transformedClassNames = processArrayExpression(arg, classNameMap);
      arg.elements = transformedClassNames;
    }

    return arg;
  });

  return args;
};
