import React from "react";
import { Pill, PillAccent } from "./Pill";

export interface SkillPillProps {
  id: string;
  name: string;
  accent?: PillAccent;
  icon?: React.ReactNode;
  onRemove?: () => void;
  onClick?: () => void;
}

export const SkillPill: React.FC<SkillPillProps> = ({
  name,
  accent = "mint",
  icon,
  onRemove,
  onClick
}) => {
  return (
    <Pill
      label={`/${name}`}
      variant="solid"
      accent={accent}
      icon={icon}
      onRemove={onRemove}
      onClick={onClick}
      className="hime-skill-pill"
    />
  );
};
