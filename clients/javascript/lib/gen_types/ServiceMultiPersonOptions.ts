// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { RoundRobinAlgorithm } from './RoundRobinAlgorithm'

export type ServiceMultiPersonOptions =
  | { variant: 'roundRobinAlgorithm'; data: RoundRobinAlgorithm }
  | { variant: 'collective' }
  | { variant: 'group'; data: number }