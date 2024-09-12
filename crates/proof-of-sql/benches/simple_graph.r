#!/usr/bin/Rscript --vanilla
args <- commandArgs(trailingOnly = TRUE)
title <- args[1]
csv_in <- args[2]
svg_out <- args[3]
if (!require(ggplot2)) {
  install.packages("ggplot2")
  library(ggplot2)
}
xscale <- scale_x_continuous()
yscale <- scale_y_continuous()
if (length(args) > 3) {
  if (args[4] == "log") {
    xscale <- scale_x_log10(breaks=c(1,10,100,1000,10000,100000,1000000,10000000,100000000),labels=c(1,10,100,1000,"10k","100k","1m","10m","100m"))
    yscale <- scale_y_log10(breaks=c(0.02,0.1,0.5,1,2,5,10,30,60,100),labels=c(0.02,0.1,0.5,1,2,5,10,30,60,100))
  }
}
dat <- read.csv(csv_in, stringsAsFactors = TRUE)
if (!require(ggdark)) {
  install.packages("ggdark")
  library(ggdark)
}
image <- ggplot(dat) +
  aes(size, time, shape = operation, fill = query_num, color = query_num) +
  stat_summary(fun = "median", geom = "line") +
  stat_summary(fun = "median", geom = "point") +
  xscale + yscale +
  scale_fill_manual(values = c("#5000bf", "#779fc6", "#CC0AAC", "#C69E76"), guide = "none") +
  scale_color_manual(values = c("#5000bf", "#779fc6", "#CC0AAC", "#C69E76"), name = NULL) +
  scale_shape_manual(values = c(21, 23), name = NULL) +
  dark_theme_minimal() + theme(panel.background = element_rect(fill = "#100217")) +
  theme(plot.title = element_text(hjust = 0.5)) +
  labs(title = "Proof of SQL Query Performance", subtitle = title, x = "Table Size (# rows)", y = "Execution Time (s)")
if (!require(svglite)) {
  install.packages("svglite")
  library(svglite)
}
ggsave(file = svg_out, plot = image, width = 16/2, height = 9/2, dpi = 200)