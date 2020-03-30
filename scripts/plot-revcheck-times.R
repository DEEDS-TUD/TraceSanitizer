#!/usr/bin/Rscript

suppressPackageStartupMessages(library('tidyverse', warn.conflicts = F))
library('magrittr', warn.conflicts = F)
library('glue', warn.conflicts = F)
library('optparse', warn.conflicts = F)
suppressPackageStartupMessages(library('viridis', warn.conflicts = F))

options(width=120)
printf = function(...) cat(sprintf(...))

# =============================================================================
input_file<- '<missing>'
# =============================================================================


cmd_options <- list()
parser <- OptionParser(usage = "%prog DATA_IN_FILE", option_list = cmd_options)
args <- parse_args(parser, positional_arguments = c(0, 1))
if(length(args$args) > 0) {
  input_file <- args$args[[1]]
}

theme_set(
  theme_minimal(base_size = 12) + theme(
    plot.margin = margin(5, 5, 5, 5),
    panel.grid.major.x = element_blank(),
    legend.position = 'right',
    legend.title = element_blank(),
    legend.margin = margin(0, 0, 0, 0),
    legend.box.spacing = unit(0, 'pt'),
    legend.text = element_text(margin = margin(l = 2, unit = 'pt')),
    legend.key.size = unit(5, 'mm'),
    axis.text.x = element_text(size = 10),
    plot.title = element_text(hjust = 0.5)
  )
)
scale_colour_discrete <- function(...) scale_color_viridis(discrete = T, ...)
scale_fill_discrete <- function(...) scale_fill_viridis(discrete = T, ...)

paper_theme <- theme_minimal(base_size = 8) + theme(
  plot.margin = margin(1, 1, 1, 1),
  panel.grid.major.x = element_blank(),
  # panel.grid.minor = element_blank(),
  legend.position = 'right',
  legend.title = element_blank(),
  legend.margin = margin(0, 0, 0, 0),
  legend.box.spacing = unit(0, 'pt'),
  legend.text = element_text(margin = margin(l = 2, r = 0, unit = 'pt')),
  legend.key.size = unit(4, 'mm'),
  axis.text.x = element_text(size = 7, angle = 25, hjust = 0.9),
  strip.text = element_text(size = 8),
  strip.background = element_rect(linetype = 'blank', fill = 'gray95'),
  plot.title = element_text(hjust = 0.5)
)

message(glue('Reading data from \'{input_file}\'...'))
csv_data <- read_csv(input_file, col_names=T) %>%
  select(Benchmark, `Solving-time`, `Total-time`) %>%
  rename(target = Benchmark, time_sol = `Solving-time`, time_tot = `Total-time`)
csv_data <- csv_data %>%
  mutate(time_fbuild = time_tot - time_sol) %>%
  gather(key = "type", value = "time", time_fbuild, time_sol) %>%
  group_by(target) %>%
  mutate(type = str_replace(type, "time_", "")) %>%
  mutate(type = str_replace(type, c("fbuild", "sol"), c("Formula", "Solver"))) %>%
  mutate(type = fct_rev(factor(type))) %>%
  select(-time_tot)

# internal plot
# csv_data %>%
#   ggplot(aes(target, time, fill=type)) +
#   geom_col()+
#   scale_fill_viridis_d(direction = -1,  end = 0.75) +
#   xlab('Program') +
#   ylab('Rev. Check Time [min]')

message('Plotting to \'plot-rev-check-times.pdf\' in current dir...')
cairo_pdf('plot-rev-check-times.pdf', width = 3, height = 1.9, pointsize = 8)
p <- csv_data %>%
  ggplot(aes(target, time, fill=type)) +
  geom_col()+
  scale_fill_viridis_d(direction = -1,  end = 0.75) +
  xlab('Program') +
  ylab('Rev. Check Time [min]') +
  paper_theme
print(p)
dev.off()

message('--Fin--')