import * as t from "@babel/types";
import { CSSModuleExports } from "lightningcss";
import { NodePath } from "@babel/traverse";

export class AttributeProcessor {
  classNameMap: CSSModuleExports;
  attrName: string;
  path: NodePath<t.JSXAttribute> | NodePath<t.CallExpression>;

  constructor(props: {
    attrName: string;
    classNameMap: CSSModuleExports;
    path: NodePath<t.JSXAttribute> | NodePath<t.CallExpression>;
  }) {
    this.attrName = props.attrName;
    this.classNameMap = props.classNameMap;
    this.path = props.path;
  }

  updateAttribute() {
    const node = (this.path as NodePath<t.JSXAttribute>).node;
    if (node.value?.type === "StringLiteral") {
      this.updateStringLiteral(node.value);
    } else if (node.value?.type === "JSXExpressionContainer") {
      if (node.value.expression.type === "CallExpression") {
        if (node.value.expression.callee.type === "MemberExpression") {
          if (
            node.value.expression.callee.object.type === "ArrayExpression" &&
            node.value.expression.callee.property.type === "Identifier" &&
            node.value.expression.callee.property.name === "join"
          ) {
            this.updateArrayExpression(node.value.expression.callee.object);
          }
        } else {
          this.updateCallExpression(node.value.expression);
        }
      } else if (node.value.expression.type === "LogicalExpression") {
        this.updateLogicalExpression(node.value.expression);
      } else if (node.value.expression.type === "ConditionalExpression") {
        this.updateConditionalExpression(node.value.expression);
      } else if (node.value.expression.type === "Identifier") {
        this.updateIdentifier(node.value.expression);
      }
    }
  }

  updateExpression = (node: t.Expression): t.Expression => {
    switch (node.type) {
      case "StringLiteral":
        return this.updateStringLiteral(node);
      case "ObjectExpression":
        return this.updateObjectExpression(node);
      case "ArrayExpression":
        return this.updateArrayExpression(node);
      case "LogicalExpression":
        return this.updateLogicalExpression(node);
      case "ConditionalExpression":
        return this.updateConditionalExpression(node);
      case "Identifier":
        return this.updateIdentifier(node);
      case "CallExpression":
        return this.updateCallExpression(node);
      case "TemplateLiteral":
        return this.updateTemplateLiteral(node);
      case "BinaryExpression":
        return this.updateBinaryExpression(node);
      default:
        return node;
    }
  };

  updateIdentifier = (node: t.Identifier) => {
    const varName = node.name;
    const binding = this.path.scope.getBinding(varName);
    if (
      binding &&
      t.isVariableDeclarator(binding.path.node) &&
      t.isExpression(binding.path.node.init)
    ) {
      this.updateExpression(binding.path.node.init);
    }
    return node;
  };

  updateStringLiteral = (node: t.StringLiteral) => {
    const updatedClassName = this.getUpdatedClassName(node.value);
    node.value = updatedClassName;
    return node;
  };

  updateBinaryExpression = (node: t.BinaryExpression) => {
    if (node.operator === "+") {
      if (t.isExpression(node.left)) {
        this.updateExpression(node.left);
      }
      if (t.isExpression(node.right)) {
        this.updateExpression(node.right);
      }
    }
    return node;
  };

  updateObjectExpression = (node: t.ObjectExpression) => {
    const properties = node.properties.map((property) => {
      if (property.type === "ObjectProperty") {
        if (t.isStringLiteral(property.key)) {
          const exportName = this.classNameMap[property.key.value];
          if (exportName) {
            return t.objectProperty(
              t.stringLiteral(exportName.name),
              property.value
            );
          }
        } else if (t.isIdentifier(property.key)) {
          this.updateIdentifier(property.key);
          return property;
        }
      }
      return property;
    });
    node.properties = properties;
    return node;
  };

  updateArrayExpression = (node: t.ArrayExpression) => {
    const elements: t.ArrayExpression["elements"] = node.elements.map(
      (element) => {
        if (t.isExpression(element)) {
          return this.updateExpression(element);
        }
        return element;
      }
    );
    node.elements = elements;
    return node;
  };

  updateLogicalExpression = (
    node: t.LogicalExpression
  ): t.LogicalExpression => {
    this.updateExpression(node.right);
    return node;
  };

  updateConditionalExpression = (node: t.ConditionalExpression) => {
    this.updateConditionalExpressionLeaf(node, "consequent");
    this.updateConditionalExpressionLeaf(node, "alternate");
    return node;
  };

  updateConditionalExpressionLeaf = (
    node: t.ConditionalExpression,
    type: "consequent" | "alternate"
  ) => {
    if (t.isExpression(node[type])) {
      this.updateExpression(node[type]);
    }
    return node;
  };

  updateCallExpression = (node: t.CallExpression) => {
    const args = node.arguments.map((arg) => {
      if (t.isExpression(arg)) {
        return this.updateExpression(arg);
      }
      return arg;
    });

    node.arguments = args;
    return node;
  };

  updateTemplateLiteral = (node: t.TemplateLiteral) => {
    node.expressions.forEach((expression) => {
      if (t.isExpression(expression)) {
        this.updateExpression(expression);
      }
    });

    node.quasis.forEach((quasi) => {
      if (quasi.type === "TemplateElement") {
        quasi.value.cooked = this.getUpdatedClassName(quasi.value.cooked);
        quasi.value.raw = this.getUpdatedClassName(quasi.value.raw);
      }
    });

    return node;
  };

  private getUpdatedClassName = <T extends string | undefined>(
    className: T
  ): T => {
    if (!className) {
      return className;
    }

    const classNames = className.split(" ");
    const transformedClassNames = classNames.map((cl) => {
      const exportName = this.classNameMap[cl]?.name;
      return exportName ?? cl;
    });

    return transformedClassNames.join(" ") as T;
  };
}
