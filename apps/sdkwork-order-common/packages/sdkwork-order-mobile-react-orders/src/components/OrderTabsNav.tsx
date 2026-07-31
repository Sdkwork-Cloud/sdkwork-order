import React, { useRef } from "react";
import { motion } from "motion/react";
import { cn } from "@sdkwork/ui-mobile-react";

interface OrderTabsNavProps {
  tabs: { id: string; label: string }[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
}

export const OrderTabsNav: React.FC<OrderTabsNavProps> = ({
  tabs,
  activeTab,
  onTabChange,
}) => {
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const handleTabClick = (
    tabId: string,
    event: React.MouseEvent<HTMLDivElement>
  ) => {
    onTabChange(tabId);
    const container = scrollContainerRef.current;
    const element = event.currentTarget;
    if (container && element) {
      const containerWidth = container.offsetWidth;
      const elementOffset = element.offsetLeft;
      const elementWidth = element.offsetWidth;
      const scrollPos = elementOffset - containerWidth / 2 + elementWidth / 2;
      container.scrollTo({ left: scrollPos, behavior: "smooth" });
    }
  };

  return (
    <div className="h-[44px] flex items-center relative border-b border-border-color/50">
      <div
        ref={scrollContainerRef}
        className="flex-1 overflow-x-auto no-scrollbar flex items-center h-full px-4 scroll-smooth"
      >
        <div className="flex gap-8 h-full items-center min-w-max">
          {tabs.map((tab) => (
            <div
              key={tab.id}
              onClick={(e) => handleTabClick(tab.id, e)}
              className="relative h-full flex items-center cursor-pointer whitespace-nowrap"
            >
              <span
                className={cn(
                  "text-[14px] transition-colors",
                  activeTab === tab.id
                    ? "font-semibold text-text-main"
                    : "font-medium text-text-sub"
                )}
              >
                {tab.label}
              </span>
              {activeTab === tab.id && (
                <div className="absolute left-0 right-0 bottom-0 flex justify-center">
                  <motion.div
                    layoutId="orderTabIndicator"
                    className="w-6 h-[3px] bg-primary-blue rounded-t-full"
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
