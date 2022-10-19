/*
 * Copyright 2022 PieDb Data
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

import { Box, Button, Flex, Text, useToast, VStack } from "@chakra-ui/react"
import { reverse, sortBy } from "lodash"
import Head from "next/head"
import Link from "next/link"
import { useRouter } from "next/router"
import { Fragment, useCallback, useEffect, useState } from "react"
import { StreamGraph } from "../components/StreamGraph"
import Title from "../components/Title"
import { ActorPoint } from "../lib/layout"
import { Table as RwTable } from "../proto/gen/catalog"
import { getMaterializedViews } from "./api/streaming"

const SIDEBAR_WIDTH = "200px"

function buildMvDependencyAsEdges(mvList: RwTable[]): ActorPoint[] {
  const edges = []
  for (const mv of reverse(sortBy(mvList, "id"))) {
    if (!mv.name.startsWith("__")) {
      edges.push({
        id: mv.id.toString(),
        name: mv.name,
        parentIds: mv.dependentRelations.map((r) => r.toString()),
        order: mv.id,
      })
    }
  }
  return edges
}

export default function StreamingGraph() {
  const toast = useToast()
  const [mvList, setMvList] = useState<RwTable[]>()

  useEffect(() => {
    async function doFetch() {
      try {
        setMvList(
          (await getMaterializedViews()).filter((x) => !x.name.startsWith("__"))
        )
      } catch (e: any) {
        toast({
          title: "Error Occurred",
          description: e.toString(),
          status: "error",
          duration: 5000,
          isClosable: true,
        })
        console.error(e)
      }
    }
    doFetch()
    return () => {}
  }, [toast])

  const mvDependencyCallback = useCallback(() => {
    if (mvList) {
      return buildMvDependencyAsEdges(mvList)
    } else {
      return undefined
    }
  }, [mvList])

  const mvDependency = mvDependencyCallback()

  const router = useRouter()

  const retVal = (
    <Flex p={3} height="calc(100vh - 20px)" flexDirection="column">
      <Title>Streaming Graph</Title>
      <Flex flexDirection="row" height="full">
        <Flex
          width={SIDEBAR_WIDTH}
          height="full"
          maxHeight="full"
          mr={3}
          alignItems="flex-start"
          flexDirection="column"
        >
          <Text fontWeight="semibold" mb={3}>
            All Nodes
          </Text>
          <Box flex={1} overflowY="scroll">
            <VStack width="full" spacing={1}>
              {mvList?.map((mv) => {
                const match = router.query.id === mv.id.toString()
                return (
                  <Link href={`?id=${mv.id}`} key={mv.id}>
                    <Button
                      colorScheme={match ? "teal" : "gray"}
                      color={match ? "teal.600" : "gray.500"}
                      variant={match ? "outline" : "ghost"}
                      width="full"
                      py={0}
                      height={8}
                      justifyContent="flex-start"
                    >
                      {mv.name}
                    </Button>
                  </Link>
                )
              })}
            </VStack>
          </Box>
        </Flex>
        <Box
          flex={1}
          height="full"
          ml={3}
          overflowX="scroll"
          overflowY="scroll"
        >
          <Text fontWeight="semibold">Graph</Text>
          {mvDependency && (
            <StreamGraph
              nodes={mvDependency}
              selectedId={router.query.id as string}
            />
          )}
        </Box>
      </Flex>
    </Flex>
  )

  return (
    <Fragment>
      <Head>
        <title>Streaming Graph</title>
      </Head>
      {retVal}
    </Fragment>
  )
}
